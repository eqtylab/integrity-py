use std::{
    fs,
    fs::File,
    path::PathBuf,
    sync::{Arc, OnceLock, RwLock},
};

use crate::indexer::{sql_indexer::IStatementIdx as IStatementIdx2, sql_lite::Sqlite};
use crate::integrity_service::Configuration as IntegrityServiceConfig;
use anyhow::{anyhow, Result};
use integrity::{
    cid::iroh::{CidIgnoreConfig, HashingConfig},
    lineage::models::statements::{Statement, StatementTrait},
    signer::SignerType,
};
use tokio::runtime::Runtime;
use uuid::Uuid;

static CTX: RwLock<Option<Context>> = RwLock::new(None);
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Get or create the global async runtime
pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create tokio runtime"))
}

use pyo3::{pyfunction, pymodule, types::PyModule, wrap_pyfunction, PyResult, Python};

use crate::to_py_err;

/// `context` submodule.
#[pymodule]
pub fn context(py: Python, m: &PyModule) -> PyResult<()> {
    let _ = py;

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
fn init(_py: Python, app_dir: PathBuf) -> PyResult<()> {
    Context::init(app_dir).map_err(to_py_err)?;
    Ok(())
}

/// Resets the global context, allowing it to be reinitialized with a new app directory
#[pyfunction]
fn reset(_py: Python) -> PyResult<()> {
    Context::reset().map_err(to_py_err)?;
    Ok(())
}

#[pyfunction]
fn set_integrity_service_url(_py: Python, url: String) -> PyResult<()> {
    Context::update_context(|ctx| ctx.integrity_service = Some(url)).map_err(to_py_err)
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
    Context::update_context(|ctx| ctx.hashing = hash_config).map_err(to_py_err)
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

    Context::update_context(|ctx| ctx.cid_ignore = cid_ignore).map_err(to_py_err)
}

#[pyfunction]
fn set_generate_model_signing_signatures(_py: Python, enable: bool) -> PyResult<()> {
    Context::update_context(|ctx| ctx.generate_model_signing_signatures = enable).map_err(to_py_err)
}

#[pyfunction]
/// Creates a graph record in the DB that statements can be registered under
fn create_graph_from_context(
    _py: Python,
    id: String,
    name: String,
    parent_id: Option<String>,
) -> PyResult<()> {
    let id = Uuid::parse_str(&id).map_err(to_py_err)?;
    let parent_id = if let Some(parent_id) = parent_id {
        let id = Uuid::parse_str(&parent_id).map_err(to_py_err)?;
        Some(id)
    } else {
        None
    };
    get_runtime()
        .block_on(ctx().sql_lite.create_graph(&id, &name, parent_id.as_ref()))
        .map_err(to_py_err)?;
    Ok(())
}

/// Gets a clone of the global application context.
///
/// # Returns
/// * `Context` - Clone of the initialized global context
///
/// # Panics
/// Panics if the global context has not been initialized via `Context::init()`.
pub fn ctx() -> Context {
    CTX.read()
        .expect("Failed to acquire read lock")
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
}

impl Context {
    /// Initializes the global application context.
    ///
    /// # Arguments
    /// * `app_dir` - Base directory for storing application data
    ///
    /// # Returns
    /// * `Result<Context>` - Initialized context, or error if initialization fails
    pub fn init(app_dir: PathBuf) -> Result<Context> {
        let mut ctx_lock = CTX
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;

        if ctx_lock.is_some() {
            log::warn!("Context already initialized. App directory was not set");
            return Ok(ctx_lock.clone().unwrap());
        }

        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)?;
        }

        let db_path = app_dir.join("statements.db");
        if !db_path.exists() {
            File::create(&db_path).map_err(to_py_err)?;
        }

        let db_path = app_dir.join("graphs.db");
        let db_init_required = !db_path.exists();
        if !db_path.exists() {
            File::create(&db_path).map_err(to_py_err)?;
        }
        let db_url = format!("sqlite:{}", db_path.display());
        let sql_lite2 = get_runtime()
            .block_on(Sqlite::new(&db_url))
            .map_err(to_py_err)?;

        if db_init_required {
            get_runtime()
                .block_on(sql_lite2.init())
                .map_err(to_py_err)?;
        }

        get_runtime()
            .block_on(sql_lite2.init())
            .map_err(to_py_err)?;

        let ctx = Context {
            app_dir,
            sql_lite: Arc::new(sql_lite2),
            hashing: Default::default(),
            cid_ignore: Default::default(),
            integrity_service: None,
            active_signer: None,
            generate_model_signing_signatures: false,
        };

        *ctx_lock = Some(ctx.clone());
        Ok(ctx)
    }

    /// Resets the global context, allowing it to be reinitialized
    pub fn reset() -> Result<()> {
        let mut ctx_lock = CTX
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;

        *ctx_lock = None;
        Ok(())
    }

    /// Updates the global context using a closure that modifies it in place.
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
        let mut ctx_lock = CTX
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;

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
            .ok_or_else(|| to_py_err("No active signer available"))?;
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

    /// Registers a statement in the local database.
    ///
    /// # Arguments
    /// * `statement` - The statement to register
    /// * `graph_id` - Graph ID to associate the statement with
    ///
    /// # Returns
    /// * `Result<()>` - Success or error if registration fails
    pub async fn register_statement_locally(
        &self,
        statement: Statement,
        graph_id: Option<&Uuid>,
    ) -> Result<()> {
        let id = statement.get_id();
        let s_type = statement.get_type_string()?;
        log::info!("registering {s_type} statement {id} to graph database");
        self.sql_lite
            .register_statement(&statement, graph_id)
            .await?;
        Ok(())
    }
}
