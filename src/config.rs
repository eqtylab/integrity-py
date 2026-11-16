use std::{
    fs,
    fs::File,
    future::Future,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, bail, Result};
use integrity::{
    blob_store::{BlobStore, LocalFs},
    cid::iroh::{CidIgnoreConfig, HashingConfig},
    lineage::models::statements::{Statement, StatementTrait},
    signer::{SignerType, VCompNotarySigner},
};
use once_cell::sync::Lazy;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, task_local};
use uuid::Uuid;

use crate::{
    indexer::{Context, Sqlite},
    CID,
};

/// Serializable settings for TOML persistence
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistentSettings {
    store_all_blobs: bool,
    cid_ignore: CidIgnoreSettings,
}

impl From<Config> for PersistentSettings {
    fn from(config: Config) -> Self {
        Self {
            store_all_blobs: config.store_all_blobs,
            cid_ignore: config.cid_ignore.clone().into(),
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

static CFG: Lazy<RwLock<Option<Config>>> = Lazy::new(|| RwLock::new(None));
#[cfg(test)]
static TEST_CONFIG_LOCK: Lazy<std::sync::Mutex<()>> = Lazy::new(|| std::sync::Mutex::new(()));
task_local! {
    static IN_WITH_CFG: bool;
}

/// Serializes tests that replace the process-wide SDK configuration.
#[cfg(test)]
pub(crate) fn lock_test_config() -> std::sync::MutexGuard<'static, ()> {
    TEST_CONFIG_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Macro to reduce boilerplate for async config operations in pyfunctions.
///
/// Usage:
/// ```rust
/// with_cfg!(py, |cfg| {
///     let graph_id = cfg.resolve_graph_id(graph_id)?;
///     // ... async operations ...
///     Ok(result)
/// })
/// ```
#[macro_export]
macro_rules! with_cfg {
    ($py:expr, |$cfg:ident| $body:expr) => {
        $py.detach(|| {
            pyo3_async_runtimes::tokio::get_runtime().block_on(async {
                $crate::config::with_cfg_guard(async {
                    let $cfg = $crate::config::cfg_async().await;
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
pub async fn cfg_async() -> Config {
    CFG.read()
        .await
        .as_ref()
        .expect("Config not initialized")
        .clone()
}

/// Runs a future while marking `with_cfg` as active for this task.
///
/// This prevents calling blocking context helpers within a `with_cfg` scope.
pub async fn with_cfg_guard<F, T>(f: F) -> T
where
    F: Future<Output = T>,
{
    IN_WITH_CFG.scope(true, f).await
}

/// Gets a clone of the global application config (blocking version).
///
/// # Returns
/// * `Result<Config>` - Clone of the initialized global config or an error if unset
pub fn cfg_blocking() -> Result<Config> {
    if IN_WITH_CFG
        .try_with(|in_with_cfg| *in_with_cfg)
        .unwrap_or(false)
    {
        return Err(anyhow!(
            "cfg_blocking() cannot be called from within with_cfg"
        ));
    }
    let cfg_lock = CFG.blocking_read();
    let cfg = cfg_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Config not initialized"))?;
    Ok(cfg.clone())
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

fn set_store_all_blobs_inner(value: bool) -> Result<()> {
    Config::update_config(|ctx| {
        ctx.store_all_blobs = value;
    })?;
    Config::save_config()?;
    Ok(())
}

/// Active signer metadata stored in the in-memory config.
///
/// Tracks both the persisted signer file name and the loaded signer payload.
#[derive(Clone)]
pub struct ActiveSigner {
    /// On-disk file name used to load this signer from the `signers/` directory.
    pub name: String,
    /// Loaded signer implementation used for signing and DID resolution.
    pub signer: SignerType,
}

/// Global application config containing configuration and state.
///
/// The config stores application-wide settings including storage directories,
/// hashing preferences, and file filtering rules.
#[derive(Clone)]
#[pyclass(from_py_object)]
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
    pub active_signer: Option<ActiveSigner>,
    /// Whether to store all blobs when computing CIDs
    pub store_all_blobs: bool,
    /// Default context used when no context is supplied.
    pub default_context: Context,
    /// Blob store used for persisted binary data.
    pub blob_store: LocalFs,
}

#[pymethods]
// Python exported impl functions
impl Config {
    // Setters
    #[pyo3(signature = (multithread=None, memory_map=None))]
    /// Updates hashing behavior used when computing CIDs.
    ///
    /// - `multithread`: Enable multithreaded hashing.
    /// - `memory_map`: Enable memory-mapped file reads where supported.
    ///
    /// Returns the updated config instance.
    fn set_hashing_config(
        &self,
        multithread: Option<bool>,
        memory_map: Option<bool>,
    ) -> PyResult<Self> {
        set_hashing_config_inner(multithread, memory_map)?;
        Ok(self.clone())
    }

    #[pyo3(signature = (include_hidden_files=None, gitignore=None, include_symlinks=None))]
    /// Updates the directory ignore rules used when computing CIDs.
    ///
    /// - `include_hidden_files`: Include hidden files in directory hashing.
    /// - `gitignore`: Respect `.gitignore` rules while hashing directories.
    /// - `include_symlinks`: Include symlinks in directory hashing.
    ///
    /// Returns the updated config instance.
    fn set_cid_ignore_rules(
        &self,
        include_hidden_files: Option<bool>,
        gitignore: Option<bool>,
        include_symlinks: Option<bool>,
    ) -> PyResult<Self> {
        set_cid_ignore_rules_inner(include_hidden_files, gitignore, include_symlinks)?;
        Ok(self.clone())
    }

    #[pyo3(signature = (value))]
    /// Sets whether blobs should be persisted automatically when computing CIDs.
    ///
    /// Returns the updated config instance.
    fn set_store_all_blobs(&self, value: bool) -> PyResult<Self> {
        set_store_all_blobs_inner(value)?;
        Ok(self.clone())
    }

    /// Returns the default context used when no context is supplied explicitly.
    fn get_default_context(&self) -> Context {
        log::debug!("Returning default context {}", self.default_context);
        self.default_context.clone()
    }
}

// Rust only impl functions
impl Config {
    /// Initializes the global application config.
    ///
    /// # Arguments
    /// * `app_dir` - Base directory for storing application data
    /// * `default_context` - A default context to register all statements under
    ///
    /// # Returns
    /// * `Result<Config>` - Initialized config, or error if initialization fails
    pub async fn init(app_dir: PathBuf, default_context: Option<Context>) -> Result<Config> {
        // Check if already initialized
        {
            let ctx_lock = CFG.read().await;
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
        if !db_path.exists() {
            File::create(&db_path)?;
        }
        let db_file = format!("sqlite:{}", db_path.display());
        let sqlite = Sqlite::new(&db_file).await?;
        // `init` uses idempotent schema statements and also applies migrations/indexes
        // for databases created by earlier SDK versions.
        sqlite.init().await?;

        let default_context = default_context.unwrap_or_default();
        log::debug!("Initializing default graph: {default_context}");
        sqlite.create_graph(&default_context).await?;

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
            store_all_blobs: persisted
                .as_ref()
                .map(|p| p.store_all_blobs)
                .unwrap_or(false),
            default_context,
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
        let mut ctx_lock = CFG.write().await;
        *ctx_lock = Some(ctx.clone());
        Ok(ctx)
    }

    /// Resets the global config, allowing it to be reinitialized (internal async version)
    pub async fn reset_internal() -> Result<()> {
        let mut ctx_lock = CFG.write().await;
        *ctx_lock = None;
        Ok(())
    }

    /// Helper fn to update the config using a closure that modifies it in place (blocking version).
    fn update_config<F>(updater: F) -> Result<()>
    where
        F: FnOnce(&mut Config),
    {
        let mut ctx_lock = CFG.blocking_write();

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
        let mut ctx_lock = CFG.write().await;

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
        Ok(signer.signer.get_did_doc().id)
    }

    /// Sets the active signer for the current config (async version).
    ///
    /// # Arguments
    /// * `signer` - The signer to set as active
    ///
    /// # Returns
    /// * `Result<()>` - Success or error if config update fails
    pub async fn set_active_signer_async(signer: SignerType, name: Option<String>) -> Result<()> {
        let active_signer_name = name.unwrap_or_else(|| signer.get_did_doc().id.clone());
        Config::update_config_async(|ctx| {
            ctx.active_signer = Some(ActiveSigner {
                name: active_signer_name,
                signer,
            });
        })
        .await
    }

    /// Resolves the Optional graph id, or the default graph id
    ///
    /// # Arguments
    /// * `Option<Context>` - Optional context object
    ///
    /// # Returns
    /// * `Result<Uuid>` - The explict or default context id
    pub fn resolve_graph_id(&self, context: Option<Context>) -> Uuid {
        match context {
            Some(g) => g.id,
            None => {
                log::trace!(
                    "Context was not provided. Using default context {:}",
                    self.default_context.id
                );
                self.default_context.id
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
        let ctx_lock = CFG.blocking_read();
        let ctx = ctx_lock
            .as_ref()
            .ok_or_else(|| anyhow!("Config not initialized"))?;

        let settings = PersistentSettings {
            store_all_blobs: ctx.store_all_blobs,
            cid_ignore: ctx.cid_ignore.clone().into(),
        };

        let config_path = ctx.app_dir.join("config.toml");
        let toml_string = toml::to_string_pretty(&settings)?;

        let mut file = File::create(&config_path)?;
        file.write_all(toml_string.as_bytes())?;

        log::debug!("Config saved to {:?}", config_path);
        Ok(())
    }
}

static NOTARY_HTTP: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

/// Notary's current signing DID via /get_dids.
async fn notary_container_did(url: &str) -> Result<String> {
    let res: serde_json::Value = NOTARY_HTTP
        .get(format!("{url}/get_dids"))
        .send()
        .await?
        .json()
        .await?;
    res.get("registeredBy")
        .and_then(|d| d.as_str())
        .map(String::from)
        .ok_or_else(|| anyhow!("notary /get_dids response missing registeredBy"))
}

/// Register a signer's DID-registration credentials and blobs into the store.
async fn register_vcomp_credentials(
    config: &Config,
    signer: &VCompNotarySigner,
) -> Result<Vec<CID>> {
    let mut ids = Vec::new();
    if let Some(statements) = &signer.credentials {
        for value in statements.values() {
            let statement: Statement = serde_json::from_value(value.clone())?;
            ids.push(CID::new(statement.get_id()));
            config
                .sql_lite
                .register_statement(&statement, &config.default_context.id)
                .await?;
        }
    }
    if let Some(blobs) = &signer.did_blobs {
        for data in blobs.values() {
            config.blob_store.put(data.clone(), 0, None).await?;
        }
    }
    Ok(ids)
}

/// Notary signer for the next statement; if the notary rotated its key, fetch and store the new credential.
pub async fn sync_active_notary_signer(
    config: &Config,
    vcomp: &VCompNotarySigner,
) -> Result<SignerType> {
    let signing_did = notary_container_did(&vcomp.url).await?;
    if signing_did == vcomp.did_doc.id {
        return Ok(SignerType::VCompNotarySigner(vcomp.clone()));
    }
    let fresh = VCompNotarySigner::create(&vcomp.url, None).await?;
    if fresh.did_doc.id != signing_did {
        bail!(
            "notary signing key {signing_did} does not match fetched credential key {}",
            fresh.did_doc.id
        );
    }
    register_vcomp_credentials(config, &fresh).await?;
    let signer = SignerType::VCompNotarySigner(fresh);
    Config::set_active_signer_async(signer.clone(), None).await?;
    Ok(signer)
}

/// Registers `statement` in `graph_id` together with a credential for it, in one transaction.
///
/// The credential is signed before the transaction begins, so the database isn't locked while
/// a notary signs. If the active signer already issued a credential for this statement, that
/// credential is reused. With `skip_proof`, only the statement is registered.
///
/// Returns the statement's ID, followed by the credential's ID unless `skip_proof` is set.
pub async fn register_statement_with_vc(
    config: &Config,
    statement: Statement,
    graph_id: Uuid,
    timestamp: Option<String>,
    skip_proof: bool,
) -> Result<Vec<CID>> {
    let statement_id = CID::new(statement.get_id());
    if skip_proof {
        config
            .sql_lite
            .register_statement(&statement, &graph_id)
            .await?;
        return Ok(vec![statement_id]);
    }

    let vc_statement = credential_for_statement(config, &statement_id, timestamp).await?;
    let vc_id = CID::new(vc_statement.get_id());
    config
        .sql_lite
        .register_statements_batch([statement, vc_statement], &graph_id)
        .await?;
    Ok(vec![statement_id, vc_id])
}

/// Returns the active signer's credential for `statement_id`, issuing one if it has none.
async fn credential_for_statement(
    config: &Config,
    statement_id: &CID,
    timestamp: Option<String>,
) -> Result<Statement> {
    use integrity::{lineage::models::statements::VcStatement, vc};

    let signer = match &config.active_signer {
        Some(ActiveSigner {
            signer: SignerType::VCompNotarySigner(vcomp),
            ..
        }) => sync_active_notary_signer(config, vcomp).await?,
        Some(active) => active.signer.clone(),
        None => bail!("An active signer is not set"),
    };

    let registered_by = signer.get_did_doc().id.clone();

    // Statement IDs are content hashes, so registering the same content again (for example
    // the same asset in another request within the same second) yields the same statement.
    // Reuse the credential this signer already issued for it instead of issuing another.
    if let Some(existing) = config
        .sql_lite
        .find_credential(&statement_id.to_string(), &registered_by)
        .await?
    {
        return Ok(existing);
    }

    let vc = vc::issue_vc(&statement_id.to_string(), signer).await?;
    let vc = serde_json::from_value(serde_json::to_value(vc)?)?;
    Ok(Statement::CredentialRegistration(
        VcStatement::create(vc, registered_by, timestamp).await?,
    ))
}

#[cfg(test)]
mod tests {
    use integrity::{lineage::models::statements::DataStatement, signer::Ed25519Signer};
    use pyo3_async_runtimes::tokio::get_runtime;
    use sqlx::Row;
    use tempfile::tempdir;

    use super::*;
    use crate::indexer::Context;

    async fn count(config: &Config, query: &str) -> i64 {
        sqlx::query(query)
            .fetch_one(config.sql_lite.pool())
            .await
            .unwrap()
            .get(0)
    }

    async fn data_statement(registered_by: &str) -> Statement {
        Statement::DataRegistration(
            DataStatement::create(
                vec!["urn:cid:input1".to_string()],
                registered_by.to_string(),
                Some("2026-11-16T12:00:00Z".to_string()),
            )
            .await
            .unwrap(),
        )
    }

    /// Runs `f` against a fresh store, with an Ed25519 signer unless `with_signer` is false.
    fn with_store(with_signer: bool, f: impl AsyncFnOnce(Config)) {
        let _config_guard = lock_test_config();
        let temp_dir = tempdir().unwrap();
        get_runtime().block_on(async {
            Config::reset_internal().await.unwrap();
            Config::init(temp_dir.path().to_path_buf(), None)
                .await
                .unwrap();
            if with_signer {
                let signer = SignerType::ED25519(Ed25519Signer::create().unwrap());
                Config::set_active_signer_async(signer, None).await.unwrap();
            }
            f(cfg_async().await).await;
            Config::reset_internal().await.unwrap();
        });
    }

    #[test]
    fn test_register_statement_with_vc_reuses_credential_for_repeated_statement() {
        with_store(true, async |config| {
            let did = config.clone().get_active_signer_did_key().unwrap();
            let first = Context::default();
            let second = Context::default();
            config.sql_lite.create_graph(&first).await.unwrap();
            config.sql_lite.create_graph(&second).await.unwrap();

            let a = register_statement_with_vc(
                &config,
                data_statement(&did).await,
                first.id,
                None,
                false,
            )
            .await
            .unwrap();
            let b = register_statement_with_vc(
                &config,
                data_statement(&did).await,
                second.id,
                None,
                false,
            )
            .await
            .unwrap();

            assert_eq!(a.len(), 2);
            let ids = |ids: &[CID]| ids.iter().map(|id| id.to_string()).collect::<Vec<_>>();
            assert_eq!(
                ids(&a),
                ids(&b),
                "the repeated statement should reuse its credential"
            );
            assert_eq!(
                count(&config, "SELECT COUNT(*) FROM credential_statements").await,
                1
            );
            let links = format!(
                "SELECT COUNT(*) FROM statement_graph_link WHERE statement_id = '{}'",
                a[1]
            );
            assert_eq!(count(&config, &links).await, 2);
        });
    }

    #[test]
    fn test_register_statement_with_vc_skip_proof_registers_only_statement() {
        with_store(true, async |config| {
            let did = config.clone().get_active_signer_did_key().unwrap();
            let graph = config.default_context.id;
            let ids =
                register_statement_with_vc(&config, data_statement(&did).await, graph, None, true)
                    .await
                    .unwrap();

            assert_eq!(ids.len(), 1);
            assert_eq!(
                count(&config, "SELECT COUNT(*) FROM data_statements").await,
                1
            );
            assert_eq!(
                count(&config, "SELECT COUNT(*) FROM credential_statements").await,
                0
            );
        });
    }

    #[test]
    fn test_register_statement_with_vc_writes_nothing_when_signing_fails() {
        with_store(false, async |config| {
            let graph = config.default_context.id;
            let result = register_statement_with_vc(
                &config,
                data_statement("did:key:tester").await,
                graph,
                None,
                false,
            )
            .await;

            assert!(result.is_err());
            assert_eq!(
                count(&config, "SELECT COUNT(*) FROM data_statements").await,
                0,
                "a statement must not be stored without the credential it was meant to have"
            );
        });
    }
}
