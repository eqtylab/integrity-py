use std::{
    fs,
    fs::File,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use crate::indexer::Graph;
use crate::indexer::Sqlite;
use crate::integrity_service::Configuration as IntegrityServiceConfig;
use anyhow::{anyhow, Context as AnyhowContext, Result};
use integrity::{
    cid::iroh::{CidIgnoreConfig, HashingConfig},
    signer::SignerType,
};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use uuid::Uuid;

static CTX: Lazy<RwLock<Option<Context>>> = Lazy::new(|| RwLock::new(None));
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Get or create the global async runtime (for legacy sync code)
pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create tokio runtime"))
}

use pyo3::prelude::*;

/// `context` submodule.
#[pymodule]
pub fn context(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(reset, m)?)?;
    m.add_function(wrap_pyfunction!(set_cid_ignore_rules, m)?)?;
    m.add_function(wrap_pyfunction!(set_integrity_service_url, m)?)?;
    m.add_function(wrap_pyfunction!(set_hashing_config, m)?)?;
    m.add_function(wrap_pyfunction!(set_generate_model_signing_signatures, m)?)?;
    m.add_function(wrap_pyfunction!(create_graph_from_context, m)?)?;

    Ok(())
}

/// Initializes the sdk context. Must be called before setting individual context values
#[pyfunction]
fn init(py: Python<'_>, app_dir: PathBuf) -> PyResult<()> {
    py.detach(|| {
        get_runtime().block_on(Context::init(app_dir))
    })?;
    Ok(())
}

/// Resets the global context, allowing it to be reinitialized with a new app directory
#[pyfunction]
fn reset(py: Python<'_>) -> PyResult<()> {
    py.detach(|| {
        get_runtime().block_on(Context::reset())
    })?;
    Ok(())
}

#[pyfunction]
fn set_integrity_service_url(_py: Python, url: String) -> PyResult<()> {
    Ok(Context::update_context(|ctx| {
        ctx.integrity_service = Some(url)
    })?)
}

#[pyfunction]
fn set_hashing_config(
    _py: Python,
    multithread: Option<bool>,
    memory_map: Option<bool>,
) -> PyResult<()> {
    let hash_config = HashingConfig {
        multithread: multithread.unwrap_or(false),
        memory_map: memory_map.unwrap_or(false),
    };
    Ok(Context::update_context(|ctx| ctx.hashing = hash_config)?)
}

#[pyfunction]
fn set_cid_ignore_rules(
    _py: Python,
    include_hidden_files: Option<bool>,
    gitignore: Option<bool>,
    include_symlinks: Option<bool>,
) -> PyResult<()> {
    let cid_ignore = CidIgnoreConfig {
        include_hidden_files: include_hidden_files
            .unwrap_or(CidIgnoreConfig::default().include_hidden_files),
        gitignore: gitignore.unwrap_or(CidIgnoreConfig::default().gitignore),
        include_symlinks: include_symlinks.unwrap_or_default(),
    };

    Ok(Context::update_context(|ctx| ctx.cid_ignore = cid_ignore)?)
}

#[pyfunction]
fn set_generate_model_signing_signatures(_py: Python, enable: bool) -> PyResult<()> {
    Ok(Context::update_context(|ctx| {
        ctx.generate_model_signing_signatures = enable
    })?)
}

#[pyfunction]
/// Creates a graph record in the DB that statements can be registered under
fn create_graph_from_context(
    py: Python<'_>,
    id: String,
    name: String,
    parent_id: Option<String>,
) -> PyResult<()> {
    let id = Uuid::parse_str(&id).context("Invalid graph ID")?;
    let parent_id = if let Some(parent_id) = parent_id {
        let id = Uuid::parse_str(&parent_id).context("Invalid parent graph ID")?;
        Some(id)
    } else {
        None
    };
    py.detach(|| {
        get_runtime().block_on(async {
            let context = ctx_async().await;
            context
                .sql_lite
                .create_graph(&id, &name, parent_id.as_ref())
                .await
        })
    })?;
    Ok(())
}

/// Gets a clone of the global application context (async version).
///
/// # Returns
/// * `Context` - Clone of the initialized global context
///
/// # Panics
/// Panics if the global context has not been initialized via `Context::init()`.
pub async fn ctx_async() -> Context {
    CTX.read()
        .await
        .as_ref()
        .expect("Context not initialized")
        .clone()
}

/// Gets a clone of the global application context (blocking version).
///
/// # Returns
/// * `Context` - Clone of the initialized global context
///
/// # Panics
/// Panics if the global context has not been initialized via `Context::init()`.
pub fn ctx() -> Context {
    CTX.blocking_read()
        .as_ref()
        .expect("Context not initialized")
        .clone()
}

/// Global application context containing configuration and state.
///
/// The context stores application-wide settings including storage directories,
/// service URLs, hashing preferences, and file filtering rules.
#[derive(Clone)]
pub struct Context {
    /// URL for the integrity service
    pub integrity_service: Option<String>,
    /// Directory to store statements, keys, etc
    pub app_dir: PathBuf,
    /// settings to change hashing features
    pub hashing: HashingConfig,
    /// Ignore list for computing CIDs of directories. List of globs for matching.
    pub cid_ignore: CidIgnoreConfig,
    /// New connection to sqlite database for storing statements
    pub sql_lite: Arc<Sqlite>,
    /// Active signer only if it has been set during the session
    pub active_signer: Option<SignerType>,
    /// Whether to generate model signing signatures when computing CIDs for directories
    pub generate_model_signing_signatures: bool,

    pub default_graph: Graph,
}

impl Context {
    /// Initializes the global application context.
    ///
    /// # Arguments
    /// * `app_dir` - Base directory for storing application data
    ///
    /// # Returns
    /// * `Result<Context>` - Initialized context, or error if initialization fails
    pub async fn init(app_dir: PathBuf) -> Result<Context> {
        // Check if already initialized
        {
            let ctx_lock = CTX.read().await;
            if ctx_lock.is_some() {
                log::warn!("Context already initialized. App directory was not set");
                return Ok(ctx_lock.clone().unwrap());
            }
        }

        // Perform filesystem and async DB operations without holding the lock
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)?;
        }

        let db_path = app_dir.join("graphs.db");
        let db_init_required = !db_path.exists();
        if !db_path.exists() {
            File::create(&db_path)?;
        }
        let db_file = format!("sqlite:{}", db_path.display());
        let sqlite = Sqlite::new(&db_file).await?;

        if db_init_required {
            sqlite.init().await?;
        }

        let ctx = Context {
            app_dir,
            sql_lite: Arc::new(sqlite),
            hashing: Default::default(),
            cid_ignore: Default::default(),
            integrity_service: None,
            active_signer: None,
            generate_model_signing_signatures: false,
            default_graph: Graph::default(),
        };

        // Acquire write lock to set the context
        let mut ctx_lock = CTX.write().await;
        *ctx_lock = Some(ctx.clone());
        Ok(ctx)
    }

    /// Resets the global context, allowing it to be reinitialized
    pub async fn reset() -> Result<()> {
        let mut ctx_lock = CTX.write().await;
        *ctx_lock = None;
        Ok(())
    }

    /// Updates the global context using a closure that modifies it in place (blocking version).
    ///
    /// # Arguments
    /// * `updater` - Closure that receives a mutable reference to the context
    ///
    /// # Returns
    /// * `Result<()>` - Success or error if context is not initialized or lock fails
    pub fn update_context<F>(updater: F) -> Result<()>
    where
        F: FnOnce(&mut Context),
    {
        let mut ctx_lock = CTX.blocking_write();

        if let Some(ctx) = ctx_lock.as_mut() {
            updater(ctx);
            Ok(())
        } else {
            Err(anyhow!("Global context is not initialized"))
        }
    }

    /// Retrieves the DID key of the currently active signer.
    ///
    /// # Returns
    /// * `Result<String>` - The DID key string, or error if no active signer is set
    pub fn get_active_signer_did_key(self) -> Result<String> {
        let signer = self
            .active_signer
            .ok_or_else(|| anyhow!("No active signer available"))?;
        Ok(signer.get_did_doc().id)
    }

    /// Sets the active signer for the current context.
    ///
    /// # Arguments
    /// * `signer` - The signer to set as active
    ///
    /// # Returns
    /// * `Result<()>` - Success or error if context update fails
    pub fn set_active_signer(&self, signer: SignerType) -> Result<()> {
        Context::update_context(|ctx| ctx.active_signer = Some(signer))
    }

    /// Creates configuration for the Integrity Service API client.
    ///
    /// # Arguments
    /// * `api_key` - Optional API key for authentication with the service
    ///
    /// # Returns
    /// * `Result<IntegrityServiceConfig>` - Configuration object for API client, or error if service URL not set
    pub fn get_integrity_service_config(
        &self,
        api_key: Option<String>,
    ) -> Result<IntegrityServiceConfig> {
        if self.integrity_service.is_none() {
            anyhow::bail!("Integrity service URL not set");
        }

        Ok(IntegrityServiceConfig {
            base_path: ctx()
                .integrity_service
                .expect("Integrity Service URL is not configured")
                .clone(),
            bearer_access_token: api_key,
            ..Default::default()
        })
    }

    /// Sets the default Graph information to be used for all statements if not explicitly set
    ///
    /// # Arguments
    /// * `graph` - The graph stuct to set as default
    ///
    /// # Returns
    /// * `Result<()>` - Sucess or error if context update fails
    pub fn set_default_graph(&self, graph: Graph) -> Result<()> {
        Context::update_context(|ctx| ctx.default_graph = graph)
    }

    /// Resolves the Optional graph id, or the default graph id
    ///
    /// # Arguments
    /// * `Option<graph_id>` - Optional graph id. MUST BE A VALID UUID
    ///
    /// # Returns
    /// * `Result<Uuid>` - The opional graph id converted to a UUID, or the default graph id
    pub fn resolve_graph_id(&self, graph_id: Option<Uuid>) -> Result<Uuid> {
        match graph_id {
            Some(id) => Ok(id),
            None => Ok(self.default_graph.id),
        }
    }
}
