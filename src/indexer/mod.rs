mod sqlite;

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use integrity::lineage::models::statements::{Statement, StatementTrait};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub use sqlite::Sqlite;
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use uuid::Uuid;

use crate::{integrity_service::Service, with_ctx};

// ============================================================================
// Graph
// ============================================================================

/// A graph structure for organizing related statements hierarchically.
///
/// Graphs group statements together with optional parent-child relationships,
/// enabling versioning and organizational structure for lineage data.
#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Graph {
    /// Unique identifier for this graph
    #[pyo3(get)]
    pub id: Uuid,
    /// Human-readable name for this graph
    #[pyo3(get)]
    pub name: String,
    /// Optional parent graph ID for hierarchical organization
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Uuid>,
    /// Statements contained in this graph (populated on retrieval)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statements: Option<Vec<Statement>>,
}

#[pymethods]
impl Graph {
    #[staticmethod]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(py: Python<'_>, name: String) -> Self {
        let graph = Graph {
            id: Uuid::new_v4(),
            name,
            parent: None,
            statements: None,
        };
        create_graph_record(py, &graph);
        graph
    }

    #[staticmethod]
    pub fn from_parent(parent: Graph) -> GraphFactory {
        GraphFactory {
            parent: Some(parent.id),
        }
    }

    #[staticmethod]
    pub fn from_uuid(py: Python<'_>, project_id: Uuid) -> PyResult<GraphFactory> {
        let graph = Graph {
            id: project_id,
            name: project_id.to_string(),
            parent: None,
            statements: None,
        };
        create_graph_record(py, &graph);
        Ok(GraphFactory {
            parent: Some(project_id),
        })
    }

    #[pyo3(signature = (service))]
    pub fn register(&self, py: Python, service: Service) -> PyResult<()> {
        log::info!("Registering graph {}", self.id);
        with_ctx!(py, |ctx| {
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

#[pyclass]
pub struct GraphFactory {
    parent: Option<Uuid>,
}

#[pymethods]
impl GraphFactory {
    #[allow(clippy::new_ret_no_self)]
    #[pyo3(signature = (name))]
    pub fn new(&self, py: Python<'_>, name: String) -> Graph {
        let graph = Graph {
            id: Uuid::new_v4(),
            name,
            parent: self.parent,
            statements: None,
        };
        create_graph_record(py, &graph);
        graph
    }
}

fn create_graph_record(py: Python<'_>, graph: &Graph) {
    with_ctx!(py, |ctx| {
        ctx.sql_lite
            .create_graph(graph)
            .await
            .expect("Failed to create record in database");
    });
}

impl Default for Graph {
    fn default() -> Self {
        let id = Uuid::new_v4();
        Graph {
            id,
            name: id.into(),
            parent: None,
            statements: None,
        }
    }
}

impl<'r> FromRow<'r, SqliteRow> for Graph {
    fn from_row(row: &'r SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let graph_id: String = row.try_get("graph_id")?;
        let name: String = row.try_get("name")?;
        let parent_id: Option<String> = row.try_get("parent_id")?;

        Ok(Graph {
            id: Uuid::parse_str(&graph_id).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            name,
            parent: parent_id
                .map(|p| Uuid::parse_str(&p))
                .transpose()
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            statements: None,
        })
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
    log::trace!("Parsing {} rows to statements", rows.len());

    for row in rows {
        let statement_row = StatementRow::from_row(&row)?;

        // Parse main statement
        let statement: Statement = serde_json::from_value(statement_row.statement)?;
        let id = statement.get_id();
        statements.insert(id, statement);

        // Parse metadata if present
        if let Some(metadata_value) = statement_row.metadata {
            if !metadata_value.is_null() {
                log::trace!("Parsing metadata");
                let metadata_statement: Statement = serde_json::from_value(metadata_value)?;
                let id = metadata_statement.get_id();
                statements.insert(id, metadata_statement);
            }
        }

        // Parse vc if present
        if let Some(vc_value) = statement_row.vc {
            if !vc_value.is_null() {
                let vc_statement: Statement = serde_json::from_value(vc_value)?;
                let id = vc_statement.get_id();
                statements.insert(id, vc_statement);
            }
        }

        // Parse did if present
        if let Some(did_value) = statement_row.did {
            if !did_value.is_null() {
                let did_statement: Statement = serde_json::from_value(did_value)?;
                let id = did_statement.get_id();
                statements.insert(id, did_statement);
            }
        }
    }
    Ok(statements)
}
