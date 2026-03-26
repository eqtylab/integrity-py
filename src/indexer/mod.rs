mod sqlite;
#[cfg(test)]
mod sqlite_tests;

use std::{collections::HashMap, env, fmt, fs::File, path::PathBuf, sync::Arc};

use anyhow::{Context as AnyhowContext, Result};
use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use integrity::{
    blob_store::BlobStore,
    cid::get_multicodec,
    lineage::models::{
        manifest::{generate_manifest, resolve_blobs, Manifest},
        statements::{Statement, StatementTrait},
    },
    signer::{load_signer as utils_load_signer, signer::SignerType},
};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub use sqlite::Sqlite;
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use uuid::Uuid;

use crate::{integrity_service::Service, signer::SIGNER_DIR, with_cfg};

// ============================================================================
// Graph Context
// ============================================================================

/// A structure for organizing related statements hierarchically in the database.
///
/// Graph context groups statements together with optional parent-child relationships,
/// enabling organizational structure for lineage graphs.
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Context {
    /// Unique identifier
    #[pyo3(get)]
    pub id: Uuid,
    /// Human-readable name
    #[pyo3(get)]
    pub name: String,
    /// Optional parent ID for hierarchical organization
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Uuid>,
}

#[pymethods]
impl Context {
    #[staticmethod]
    #[allow(clippy::new_ret_no_self)]
    /// Creates a new context with the given name.
    ///
    /// If the global config is initialized, the context is persisted to sqlite.
    pub fn new(py: Python<'_>, name: String) -> Self {
        let context = Context {
            id: Uuid::new_v4(),
            name,
            parent: None,
        };
        maybe_create_graph_in_db(py, &context);
        context
    }

    #[staticmethod]
    /// Returns a factory that creates contexts with the provided parent.
    pub fn with_parent(parent: Context) -> ContextFactory {
        ContextFactory {
            parent: Some(parent.id),
        }
    }

    #[staticmethod]
    /// Creates a root context with the given UUID.
    pub fn from_uuid(py: Python<'_>, project_id: Uuid) -> Context {
        let context = Context {
            id: project_id,
            name: project_id.to_string(),
            parent: None,
        };
        maybe_create_graph_in_db(py, &context);
        context
    }

    #[pyo3(signature = (service))]
    /// Registers this context, its ancestors, statements, and blobs with a service.
    pub fn register(&self, py: Python, service: Service) -> PyResult<()> {
        log::info!("Registering context {}", self.id);
        with_cfg!(py, |ctx| {
            let graph_id = self.id;
            let sql_client = ctx.sql_lite;

            log::info!("Retrieving graphs {graph_id:?}");

            let graphs = sql_client.get_graph_ancestors(&graph_id).await?;
            service.register_graphs(graphs).await?;

            let statements = sql_client.retrieve_statements(&graph_id).await?;
            let blob_map = integrity::lineage::models::manifest::resolve_blobs(
                &statements,
                Arc::new(ctx.blob_store.clone()),
                8,
            )
            .await?;
            service.register_blobs(blob_map).await?;

            service.register_statements(graph_id, statements).await?;
            Ok::<(), anyhow::Error>(())
        })?;

        Ok(())
    }

    fn delete_tree(&self, py: Python<'_>) -> PyResult<()> {
        with_cfg!(py, |ctx| {
            ctx.sql_lite.delete_graph_tree(&self.id).await?;
            Ok::<_, anyhow::Error>(())
        })?;
        Ok(())
    }

    fn delete(&self, py: Python<'_>) -> PyResult<()> {
        with_cfg!(py, |ctx| {
            ctx.sql_lite.delete_graph_no_children(&self.id).await?;
            Ok::<_, anyhow::Error>(())
        })?;
        Ok(())
    }

    #[pyo3(signature = (path))]
    /// Exports this context's statements and blobs to a manifest JSON file.
    pub fn export(&self, py: Python, path: PathBuf) -> PyResult<()> {
        log::info!("Exporting {}", self.id);
        with_cfg!(py, |cfg| {
            let graph_id = self.id;
            let sql_client = cfg.sql_lite;

            let statements = sql_client.retrieve_statements(&graph_id).await?;

            let blob_store = Arc::new(cfg.blob_store.clone());
            let mut blobs = resolve_blobs(&statements, blob_store, 8).await?;

            let include_context = env::var("EQTY_INCLUDE_MANIFEST_CONTEXT")
                .map(|v| v.to_lowercase() != "false")
                .unwrap_or(true);
            log::debug!("including manifest context: {include_context}");

            if let Some(active_signer) = cfg.active_signer.as_ref() {
                let signer_path = cfg.app_dir.join(SIGNER_DIR).join(&active_signer.name);
                if signer_path.exists() {
                    let signer = utils_load_signer(signer_path)?;
                    if let SignerType::VCompNotarySigner(saved_signer) = signer {
                        if let Some(did_blobs) = saved_signer.did_blobs {
                            blobs.extend(
                                did_blobs
                                    .into_iter()
                                    .map(|(cid, data)| (cid, BASE64.encode(data))),
                            );
                        }
                    }
                }
            }

            let manifest = generate_manifest(include_context, statements, blobs).await?;

            let file = File::create(&path)
                .map_err(|e| anyhow::anyhow!("Failed to create manifest file: {e}"))?;

            serde_json::to_writer(&file, &manifest)
                .context("Failed to serialize manifest to file")?;

            log::info!("Manifest exported to {}", path.display());

            Ok::<_, anyhow::Error>(())
        })?;
        Ok(())
    }

    #[pyo3(name = "import_manifest", signature = (path))]
    /// Imports the statements and blobs from a manifest file to this context.
    pub fn import_manifest(&self, py: Python, path: PathBuf) -> PyResult<()> {
        log::info!("importing manifest {}", self.id);
        with_cfg!(py, |ctx| {
            let file = File::open(&path)
                .map_err(|e| anyhow::anyhow!("Failed to open manifest file: {e}"))?;
            let manifest: Manifest = serde_json::from_reader(file).context(format!(
                "Failed to deserialize manifest from file: {}",
                path.clone().display()
            ))?;

            log::debug!(
                "Importing manifest version {} with {} statements and {} blobs into graph {}",
                manifest.version,
                manifest.statements.len(),
                manifest.blobs.len(),
                self.id
            );

            if !manifest.contexts.is_empty() {
                log::debug!(
                    "Manifest contains {} embedded context(s); these are not persisted separately",
                    manifest.contexts.len()
                );
            }

            ctx.sql_lite.create_graph(self).await?;

            for (cid, blob_base64) in manifest.blobs {
                let blob = BASE64
                    .decode(blob_base64)
                    .with_context(|| format!("Failed to decode manifest blob '{cid}'"))?;

                let codec = get_multicodec(&cid)
                    .with_context(|| format!("Failed to determine multicodec for blob '{cid}'"))?;

                ctx.blob_store
                    .put(blob, codec, Some(&cid))
                    .await
                    .with_context(|| format!("Failed to store manifest blob '{cid}'"))?;
            }

            for statement in manifest.statements.into_values() {
                let statement_id = statement.get_id();
                ctx.sql_lite
                    .register_statement(&statement, &self.id)
                    .await
                    .with_context(|| {
                        format!("Failed to register manifest statement '{statement_id}'")
                    })?;
            }

            log::info!("Manifest imported from {}", path.display());
            Ok::<_, anyhow::Error>(())
        })?;
        Ok(())
    }

    fn __str__(&self) -> String {
        match self.parent {
            Some(parent) => format!(
                "Context(id={}, name={}, parent={})",
                self.id, self.name, parent
            ),
            None => format!("Context(id={}, name={})", self.id, self.name),
        }
    }
}

/// Factory for creating contexts with an optional parent.
#[pyclass]
pub struct ContextFactory {
    parent: Option<Uuid>,
}

#[pymethods]
impl ContextFactory {
    #[allow(clippy::new_ret_no_self)]
    #[pyo3(signature = (name))]
    /// Creates a new context using the factory's parent if set.
    pub fn new(&self, py: Python<'_>, name: String) -> Context {
        let context = Context {
            id: Uuid::new_v4(),
            name,
            parent: self.parent,
        };
        log::debug!("Creating new context: {:?}", context.__str__());
        maybe_create_graph_in_db(py, &context);
        context
    }
}

impl Default for Context {
    fn default() -> Self {
        let id = Uuid::new_v4();
        Context {
            id,
            name: id.into(),
            parent: None,
        }
    }
}

impl<'r> FromRow<'r, SqliteRow> for Context {
    fn from_row(row: &'r SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let graph_id: String = row.try_get("graph_id")?;
        let name: String = row.try_get("name")?;
        let parent_id: Option<String> = row.try_get("parent_id")?;

        Ok(Context {
            id: Uuid::parse_str(&graph_id).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            name,
            parent: parent_id
                .map(|p| Uuid::parse_str(&p))
                .transpose()
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
        })
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.__str__())
    }
}

// ============================================================================
// Row Types
// ============================================================================

/// Database row representing a statement with optional metadata and credentials.
#[derive(Debug)]
pub(crate) struct StatementRow {
    pub statement: Value,
    pub metadata: Option<Value>,
    pub vc: Option<Value>,
    pub did: Option<Value>,
}

impl<'r> FromRow<'r, SqliteRow> for StatementRow {
    fn from_row(row: &'r SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        fn parse_json(s: String) -> std::result::Result<Value, sqlx::Error> {
            serde_json::from_str(&s).map_err(|e| sqlx::Error::Decode(Box::new(e)))
        }

        fn parse_optional_json(
            s: Option<String>,
        ) -> std::result::Result<Option<Value>, sqlx::Error> {
            s.map(parse_json).transpose()
        }

        Ok(StatementRow {
            statement: parse_json(row.try_get("statement")?)?,
            metadata: parse_optional_json(row.try_get("metadata")?)?,
            vc: parse_optional_json(row.try_get("vc")?)?,
            did: parse_optional_json(row.try_get("did")?)?,
        })
    }
}

// ============================================================================
// Row Parsing
// ============================================================================

/// Parses SQLite rows into statements
pub(crate) fn rows_to_statements(rows: Vec<SqliteRow>) -> Result<HashMap<String, Statement>> {
    let mut statements = HashMap::new();
    log::debug!("Parsing {} rows to statements", rows.len());

    for row in rows {
        let statement_row = StatementRow::from_row(&row)?;

        // Parse main statement
        let statement: Statement = serde_json::from_value(statement_row.statement)?;
        let id = statement.get_id();
        statements.insert(id, statement);

        // Parse metadata if present
        if let Some(metadata_value) = statement_row.metadata {
            if !metadata_value.is_null() {
                log::debug!("Parsing metadata");
                let metadata_statement: Statement = serde_json::from_value(metadata_value)?;
                let id = metadata_statement.get_id();
                statements.insert(id, metadata_statement);
            }
        }

        // Parse vc if present
        if let Some(vc_value) = statement_row.vc {
            if !vc_value.is_null() {
                log::debug!("Parsing VC");
                let vc_statement: Statement = serde_json::from_value(vc_value)?;
                let id = vc_statement.get_id();
                statements.insert(id, vc_statement);
            }
        }

        // Parse did if present
        if let Some(did_value) = statement_row.did {
            if !did_value.is_null() {
                log::debug!("Parsing did");
                let did_statement: Statement = serde_json::from_value(did_value)?;
                let id = did_statement.get_id();
                statements.insert(id, did_statement);
            }
        }
    }
    Ok(statements)
}

// ContextFactory::new may be called before sdk initalization
// If we are initalized, save the graph to the db, otherwise .init() will save the graph
fn maybe_create_graph_in_db(py: Python<'_>, context: &Context) {
    if let Ok(ctx) = crate::config::cfg_blocking() {
        let _ = py.detach(|| {
            pyo3_async_runtimes::tokio::get_runtime().block_on(ctx.sql_lite.create_graph(context))
        });
    }
}
