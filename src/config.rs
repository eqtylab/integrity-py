use std::{
    fs,
    fs::File,
    future::Future,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use integrity::{
    blob_store::{BlobStore, LocalFs},
    cid::iroh::{CidIgnoreConfig, HashingConfig},
    signer::SignerType,
};
use once_cell::sync::Lazy;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, task_local};
use uuid::Uuid;

use crate::indexer::{Graph, Sqlite};

/// Serializable settings for TOML persistence
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistentSettings {
    store_all_blobs: bool,
    cid_ignore: CidIgnoreSettings,
    generate_model_signing_signatures: bool,
}

impl From<Config> for PersistentSettings {
    fn from(config: Config) -> Self {
        Self {
            store_all_blobs: config.store_all_blobs,
            cid_ignore: config.cid_ignore.clone().into(),
            generate_model_signing_signatures: config.generate_model_signing_signatures,
        }
    }
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
task_local! {
    static IN_WITH_CTX: bool;
}

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
                $crate::config::with_ctx_guard(async {
                    let $ctx = $crate::config::ctx_async().await;
                    $body
                })
                .await
            })
        })
    };
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

pub async fn with_ctx_guard<F, T>(f: F) -> T
where
    F: Future<Output = T>,
{
    IN_WITH_CTX.scope(true, f).await
}

pub fn ctx_blocking() -> Result<Config> {
    if IN_WITH_CTX
        .try_with(|in_with_ctx| *in_with_ctx)
        .unwrap_or(false)
    {
        return Err(anyhow!(
            "ctx_blocking() cannot be called from within with_ctx"
        ));
    }
    let ctx_lock = CTX.blocking_read();
    let ctx = ctx_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Config not initialized"))?;
    Ok(ctx.clone())
}

fn set_hashing_config_inner(multithread: Option<bool>, memory_map: Option<bool>) -> Result<()> {
    let hash_config = HashingConfig {
        multithread: multithread.unwrap_or(false),
        memory_map: memory_map.unwrap_or(false),
    };
    Config::update_config(|ctx| ctx.hashing = hash_config)?;
    Ok(())
}

fn set_cid_ignore_rules_inner(
    include_hidden_files: Option<bool>,
    gitignore: Option<bool>,
    include_symlinks: Option<bool>,
) -> Result<()> {
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

fn set_generate_model_signing_signatures_inner(enable: bool) -> Result<()> {
    Config::update_config(|ctx| {
        ctx.generate_model_signing_signatures = enable;
    })?;
    Config::save_config()?;
    Ok(())
}

fn set_store_all_blobs_inner(value: bool) -> Result<()> {
    Config::update_config(|ctx| {
        ctx.store_all_blobs = value;
    })?;
    Config::save_config()?;
    Ok(())
}

/// Global application config containing configuration and state.
///
/// The config stores application-wide settings including storage directories,
/// hashing preferences, and file filtering rules.
#[derive(Clone)]
#[pyclass]
pub struct Config {
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
    pub blob_store: LocalFs,
}

#[pymethods]
// Python exported impl functions
impl Config {
    // Setters

    #[pyo3(signature = (multithread=None, memory_map=None))]
    fn set_hashing_config(
        &self,
        multithread: Option<bool>,
        memory_map: Option<bool>,
    ) -> PyResult<Self> {
        set_hashing_config_inner(multithread, memory_map)?;
        Ok(self.clone())
    }

    #[pyo3(signature = (include_hidden_files=None, gitignore=None, include_symlinks=None))]
    fn set_cid_ignore_rules(
        &self,
        include_hidden_files: Option<bool>,
        gitignore: Option<bool>,
        include_symlinks: Option<bool>,
    ) -> PyResult<Self> {
        set_cid_ignore_rules_inner(include_hidden_files, gitignore, include_symlinks)?;
        Ok(self.clone())
    }

    #[pyo3(signature = (enable))]
    fn set_generate_model_signing_signatures(&self, enable: bool) -> PyResult<Self> {
        set_generate_model_signing_signatures_inner(enable)?;
        Ok(self.clone())
    }

    #[pyo3(signature = (value))]
    fn set_store_all_blobs(&self, value: bool) -> PyResult<Self> {
        set_store_all_blobs_inner(value)?;
        Ok(self.clone())
    }
}

// Rust only impl functions
impl Config {
    /// Initializes the global application config.
    ///
    /// # Arguments
    /// * `app_dir` - Base directory for storing application data
    ///
    /// # Returns
    /// * `Result<Config>` - Initialized config, or error if initialization fails
    pub async fn init(app_dir: PathBuf, default_graph: Option<Graph>) -> Result<Config> {
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
        let mut blob_store = LocalFs::new(app_dir.join("blobs"));
        blob_store.init().await?;

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

        let default_graph = default_graph.unwrap_or_default();
        log::debug!("Initializing default graph: {default_graph}");
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
            blob_store,
        };

        if persisted.is_none() {
            let config_path = ctx.app_dir.join("config.toml");
            let toml_string = toml::to_string_pretty::<PersistentSettings>(&ctx.clone().into())?;

            let mut file = File::create(&config_path)?;
            file.write_all(toml_string.as_bytes())?;

            log::debug!("Config saved to {:?}", config_path);
        }

        // Acquire write lock to set the config
        let mut ctx_lock = CTX.write().await;
        *ctx_lock = Some(ctx.clone());
        Ok(ctx)
    }

    /// Resets the global config, allowing it to be reinitialized (internal async version)
    pub async fn reset_internal() -> Result<()> {
        let mut ctx_lock = CTX.write().await;
        *ctx_lock = None;
        Ok(())
    }

    /// Helper fn to update the config using a closure that modifies it in place (blocking version).
    fn update_config<F>(updater: F) -> Result<()>
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

    /// Helper fn to update the config using a closure that modifies it in place (async version).
    async fn update_config_async<F>(updater: F) -> Result<()>
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
    fn save_config() -> Result<()> {
        let ctx_lock = CTX.blocking_read();
        let ctx = ctx_lock
            .as_ref()
            .ok_or_else(|| anyhow!("Config not initialized"))?;

        let settings = PersistentSettings {
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

/// Creates a VC statement for the given statement id and registers it to sqlite
pub async fn create_vc_for_statement(
    config: &Config,
    statement_id: &str,
    graph_id: Uuid,
    timestamp: Option<String>,
) -> Result<String> {
    use integrity::{
        lineage::models::statements::{Statement, StatementTrait, VcStatement},
        vc,
    };

    let signer = config
        .active_signer
        .clone()
        .ok_or_else(|| anyhow!("An active signer is not set"))?;

    let registered_by = signer.get_did_doc().id.clone();
    let vc = vc::issue_vc(statement_id, signer).await?;
    let vc_statement =
        Statement::CredentialRegistration(VcStatement::create(vc, registered_by, timestamp).await?);
    let vc_id = vc_statement.get_id();

    config
        .sql_lite
        .register_statement(&vc_statement, &graph_id)
        .await?;

    Ok(vc_id)
}
