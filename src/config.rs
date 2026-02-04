use std::{
    fs,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

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
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Serializable settings for TOML persistence
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistentSettings {
    url: Option<String>,
    store_all_blobs: bool,
    cid_ignore: CidIgnoreSettings,
    generate_model_signing_signatures: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CidIgnoreSettings {
    include_hidden_files: bool,
    gitignore: bool,
    include_symlinks: bool,
}

impl From<CidIgnoreConfig> for CidIgnoreSettings {
    fn from(config: CidIgnoreConfig) -> Self {
        Self {
            include_hidden_files: config.include_hidden_files,
            gitignore: config.gitignore,
            include_symlinks: config.include_symlinks,
        }
    }
}

impl From<CidIgnoreSettings> for CidIgnoreConfig {
    fn from(settings: CidIgnoreSettings) -> Self {
        Self {
            include_hidden_files: settings.include_hidden_files,
            gitignore: settings.gitignore,
            include_symlinks: settings.include_symlinks,
        }
    }
}

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
    // Setters
    m.add_function(wrap_pyfunction!(set_cid_ignore_rules, m)?)?;
    m.add_function(wrap_pyfunction!(set_integrity_service_url, m)?)?;
    m.add_function(wrap_pyfunction!(set_hashing_config, m)?)?;
    m.add_function(wrap_pyfunction!(set_generate_model_signing_signatures, m)?)?;
    m.add_function(wrap_pyfunction!(set_default_graph, m)?)?;
    m.add_function(wrap_pyfunction!(set_store_all_blobs, m)?)?;
    // Getters
    m.add_function(wrap_pyfunction!(get_integrity_service_url, m)?)?;
    m.add_function(wrap_pyfunction!(get_store_all_blobs, m)?)?;
    m.add_function(wrap_pyfunction!(get_cid_ignore_rules, m)?)?;
    m.add_function(wrap_pyfunction!(get_generate_model_signing_signatures, m)?)?;
    m.add_function(wrap_pyfunction!(get_app_dir, m)?)?;
    m.add_function(wrap_pyfunction!(get_blob_dir, m)?)?;
    m.add_function(wrap_pyfunction!(get_default_graph, m)?)?;

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
    Config::update_config(|ctx| {
        ctx.integrity_service = Some(url);
    })?;
    Config::save_config()?;
    Ok(())
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

    Config::update_config(|ctx| ctx.cid_ignore = cid_ignore)?;
    Config::save_config()?;
    Ok(())
}

#[pyfunction]
fn set_generate_model_signing_signatures(_py: Python, enable: bool) -> PyResult<()> {
    Config::update_config(|ctx| {
        ctx.generate_model_signing_signatures = enable;
    })?;
    Config::save_config()?;
    Ok(())
}

#[pyfunction]
fn set_store_all_blobs(_py: Python, value: bool) -> PyResult<()> {
    Config::update_config(|ctx| {
        ctx.store_all_blobs = value;
    })?;
    Config::save_config()?;
    Ok(())
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

// Getter functions exposed to Python

#[pyfunction]
fn get_integrity_service_url(_py: Python) -> PyResult<Option<String>> {
    let ctx_lock = CTX.blocking_read();
    let ctx = ctx_lock
        .as_ref()
        .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Config not initialized"))?;
    Ok(ctx.integrity_service.clone())
}

#[pyfunction]
fn get_store_all_blobs(_py: Python) -> PyResult<bool> {
    let ctx_lock = CTX.blocking_read();
    let ctx = ctx_lock
        .as_ref()
        .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Config not initialized"))?;
    Ok(ctx.store_all_blobs)
}

#[pyfunction]
fn get_cid_ignore_rules(_py: Python) -> PyResult<(bool, bool, bool)> {
    let ctx_lock = CTX.blocking_read();
    let ctx = ctx_lock
        .as_ref()
        .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Config not initialized"))?;
    Ok((
        ctx.cid_ignore.include_hidden_files,
        ctx.cid_ignore.gitignore,
        ctx.cid_ignore.include_symlinks,
    ))
}

#[pyfunction]
fn get_generate_model_signing_signatures(_py: Python) -> PyResult<bool> {
    let ctx_lock = CTX.blocking_read();
    let ctx = ctx_lock
        .as_ref()
        .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Config not initialized"))?;
    Ok(ctx.generate_model_signing_signatures)
}

#[pyfunction]
fn get_app_dir(_py: Python) -> PyResult<PathBuf> {
    let ctx_lock = CTX.blocking_read();
    let ctx = ctx_lock
        .as_ref()
        .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Config not initialized"))?;
    Ok(ctx.app_dir.clone())
}

#[pyfunction]
fn get_blob_dir(_py: Python) -> PyResult<PathBuf> {
    let ctx_lock = CTX.blocking_read();
    let ctx = ctx_lock
        .as_ref()
        .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Config not initialized"))?;
    let blob_dir = ctx.app_dir.join("blobs");
    if !blob_dir.exists() {
        fs::create_dir_all(&blob_dir).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to create blob directory: {}", e))
        })?;
    }
    Ok(blob_dir)
}

#[pyfunction]
fn get_default_graph(_py: Python) -> PyResult<Graph> {
    let ctx_lock = CTX.blocking_read();
    let ctx = ctx_lock
        .as_ref()
        .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Config not initialized"))?;
    Ok(ctx.default_graph.clone())
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
    /// Whether to store all blobs when computing CIDs
    pub store_all_blobs: bool,

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

        // Load persisted settings if config.toml exists
        let persisted = Self::load_config(&app_dir);

        let ctx = Config {
            app_dir,
            sql_lite: Arc::new(sqlite),
            hashing: Default::default(),
            cid_ignore: persisted
                .as_ref()
                .map(|p| p.cid_ignore.clone().into())
                .unwrap_or_default(),
            integrity_service: persisted.as_ref().and_then(|p| p.url.clone()),
            active_signer: None,
            generate_model_signing_signatures: persisted
                .as_ref()
                .map(|p| p.generate_model_signing_signatures)
                .unwrap_or(false),
            store_all_blobs: persisted
                .as_ref()
                .map(|p| p.store_all_blobs)
                .unwrap_or(false),
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

    /// Loads persisted settings from config.toml if it exists
    fn load_config(app_dir: &Path) -> Option<PersistentSettings> {
        let config_path = app_dir.join("config.toml");
        if !config_path.exists() {
            return None;
        }

        let mut file = match File::open(&config_path) {
            Ok(f) => f,
            Err(e) => {
                log::warn!("Failed to open config file: {}", e);
                return None;
            }
        };

        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents) {
            log::warn!("Failed to read config file: {}", e);
            return None;
        }

        match toml::from_str(&contents) {
            Ok(settings) => Some(settings),
            Err(e) => {
                log::warn!("Failed to parse config file: {}", e);
                None
            }
        }
    }

    /// Saves current config settings to config.toml
    pub fn save_config() -> Result<()> {
        let ctx_lock = CTX.blocking_read();
        let ctx = ctx_lock
            .as_ref()
            .ok_or_else(|| anyhow!("Config not initialized"))?;

        let settings = PersistentSettings {
            url: ctx.integrity_service.clone(),
            store_all_blobs: ctx.store_all_blobs,
            cid_ignore: ctx.cid_ignore.clone().into(),
            generate_model_signing_signatures: ctx.generate_model_signing_signatures,
        };

        let config_path = ctx.app_dir.join("config.toml");
        let toml_string = toml::to_string_pretty(&settings)?;

        let mut file = File::create(&config_path)?;
        file.write_all(toml_string.as_bytes())?;

        log::debug!("Config saved to {:?}", config_path);
        Ok(())
    }
}
