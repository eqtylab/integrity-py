use std::{fs, fs::File, path::PathBuf, sync::Arc};

use crate::indexer::Graph;
use crate::indexer::Sqlite;
use crate::integrity_service::Configuration as IntegrityServiceConfig;
use anyhow::{anyhow, Result};
use integrity::{
    cid::iroh::{CidIgnoreConfig, HashingConfig},
    signer::SignerType,
};
use once_cell::sync::Lazy;
use pyo3_async_runtimes::tokio::get_runtime;
use tokio::sync::RwLock;
use uuid::Uuid;

static CTX: Lazy<RwLock<Option<Config>>> = Lazy::new(|| RwLock::new(None));

use pyo3::prelude::*;

/// Macro to reduce boilerplate for async config operations in pyfunctions.
///
/// Usage:
/// ```rust
/// with_ctx!(py, |ctx| {
///     let graph_id = ctx.resolve_graph_id(graph_id)?;
///     // ... async operations ...
///     Ok(result)
/// })
/// ```
#[macro_export]
macro_rules! with_ctx {
    ($py:expr, |$ctx:ident| $body:expr) => {
        $py.detach(|| {
            pyo3_async_runtimes::tokio::get_runtime().block_on(async {
                let $ctx = $crate::config::ctx_async().await;
                $body
            })
        })
    };
}

/// `config` submodule.
#[pymodule]
pub fn config(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(reset, m)?)?;
    m.add_function(wrap_pyfunction!(set_cid_ignore_rules, m)?)?;
    m.add_function(wrap_pyfunction!(set_integrity_service_url, m)?)?;
    m.add_function(wrap_pyfunction!(set_hashing_config, m)?)?;
    m.add_function(wrap_pyfunction!(set_generate_model_signing_signatures, m)?)?;
    m.add_function(wrap_pyfunction!(set_default_graph, m)?)?;

    Ok(())
}

/// Initializes the sdk config. Must be called before setting individual config values
#[pyfunction]
fn init(py: Python<'_>, app_dir: PathBuf) -> PyResult<()> {
    py.detach(|| get_runtime().block_on(Config::init(app_dir)))?;
    Ok(())
}

/// Resets the global config, allowing it to be reinitialized with a new app directory
#[pyfunction]
fn reset(py: Python<'_>) -> PyResult<()> {
    py.detach(|| get_runtime().block_on(Config::reset()))?;
    Ok(())
}

#[pyfunction]
fn set_integrity_service_url(_py: Python, url: String) -> PyResult<()> {
    Ok(Config::update_config(|ctx| {
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
    Ok(Config::update_config(|ctx| ctx.hashing = hash_config)?)
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

    Ok(Config::update_config(|ctx| ctx.cid_ignore = cid_ignore)?)
}

#[pyfunction]
fn set_generate_model_signing_signatures(_py: Python, enable: bool) -> PyResult<()> {
    Ok(Config::update_config(|ctx| {
        ctx.generate_model_signing_signatures = enable
    })?)
}

#[pyfunction]
/// Sets the default graph context
fn set_default_graph(py: Python, graph: Graph) -> PyResult<()> {
    with_ctx!(py, |ctx| {
        log::info!("Setting default graph: {graph:?}");
        let _ = ctx.sql_lite.create_graph(&graph).await;
    });

    Ok(Config::update_config(|ctx| ctx.default_graph = graph)?)
}

/// Gets a clone of the global application config (async version).
///
/// # Returns
/// * `Config` - Clone of the initialized global config
///
/// # Panics
/// Panics if the global config has not been initialized via `Config::init()`.
pub async fn ctx_async() -> Config {
    CTX.read()
        .await
        .as_ref()
        .expect("Config not initialized")
        .clone()
}

/// Global application config containing configuration and state.
///
/// The config stores application-wide settings including storage directories,
/// service URLs, hashing preferences, and file filtering rules.
#[derive(Clone)]
pub struct Config {
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

impl Config {
    /// Initializes the global application config.
    ///
    /// # Arguments
    /// * `app_dir` - Base directory for storing application data
    ///
    /// # Returns
    /// * `Result<Config>` - Initialized config, or error if initialization fails
    pub async fn init(app_dir: PathBuf) -> Result<Config> {
        // Check if already initialized
        {
            let ctx_lock = CTX.read().await;
            if ctx_lock.is_some() {
                log::warn!("Config already initialized. App directory was not set");
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

        let default_graph = Graph::default();
        sqlite.create_graph(&default_graph).await?;
        let ctx = Config {
            app_dir,
            sql_lite: Arc::new(sqlite),
            hashing: Default::default(),
            cid_ignore: Default::default(),
            integrity_service: None,
            active_signer: None,
            generate_model_signing_signatures: false,
            default_graph,
        };

        // Acquire write lock to set the config
        let mut ctx_lock = CTX.write().await;
        *ctx_lock = Some(ctx.clone());
        Ok(ctx)
    }

    /// Resets the global config, allowing it to be reinitialized
    pub async fn reset() -> Result<()> {
        let mut ctx_lock = CTX.write().await;
        *ctx_lock = None;
        Ok(())
    }

    /// Updates the global config using a closure that modifies it in place (blocking version).
    ///
    /// # Arguments
    /// * `updater` - Closure that receives a mutable reference to the config
    ///
    /// # Returns
    /// * `Result<()>` - Success or error if config is not initialized or lock fails
    pub fn update_config<F>(updater: F) -> Result<()>
    where
        F: FnOnce(&mut Config),
    {
        let mut ctx_lock = CTX.blocking_write();

        if let Some(ctx) = ctx_lock.as_mut() {
            updater(ctx);
            Ok(())
        } else {
            Err(anyhow!("Global config is not initialized"))
        }
    }

    /// Updates the global config using a closure that modifies it in place (async version).
    ///
    /// # Arguments
    /// * `updater` - Closure that receives a mutable reference to the config
    ///
    /// # Returns
    /// * `Result<()>` - Success or error if config is not initialized or lock fails
    pub async fn update_config_async<F>(updater: F) -> Result<()>
    where
        F: FnOnce(&mut Config),
    {
        let mut ctx_lock = CTX.write().await;

        if let Some(ctx) = ctx_lock.as_mut() {
            updater(ctx);
            Ok(())
        } else {
            Err(anyhow!("Global config is not initialized"))
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

    /// Sets the active signer for the current config.
    ///
    /// # Arguments
    /// * `signer` - The signer to set as active
    ///
    /// # Returns
    /// * `Result<()>` - Success or error if config update fails
    pub fn set_active_signer(&self, signer: SignerType) -> Result<()> {
        Config::update_config(|ctx| ctx.active_signer = Some(signer))
    }

    /// Sets the active signer for the current config (async version).
    ///
    /// # Arguments
    /// * `signer` - The signer to set as active
    ///
    /// # Returns
    /// * `Result<()>` - Success or error if config update fails
    pub async fn set_active_signer_async(signer: SignerType) -> Result<()> {
        Config::update_config_async(|ctx| ctx.active_signer = Some(signer)).await
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
        let base_path = self
            .integrity_service
            .clone()
            .ok_or_else(|| anyhow!("Integrity service URL not set"))?;

        Ok(IntegrityServiceConfig {
            base_path,
            bearer_access_token: api_key,
            ..Default::default()
        })
    }

    /// Resolves the Optional graph id, or the default graph id
    ///
    /// # Arguments
    /// * `Option<graph_id>` - Optional graph id. MUST BE A VALID UUID
    ///
    /// # Returns
    /// * `Result<Uuid>` - The opional graph id converted to a UUID, or the default graph id
    pub fn resolve_graph_id(&self, graph_id: Option<Uuid>) -> Uuid {
        match graph_id {
            Some(id) => id,
            None => {
                log::trace!(
                    "GraphID was not provided. Using default graph {:}",
                    self.default_graph.id
                );
                self.default_graph.id
            }
        }
    }
}
