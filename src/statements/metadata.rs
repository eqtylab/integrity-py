use anyhow::{anyhow, Context as AnyhowContext};
use integrity::lineage::models::statements::{MetadataStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};
use serde_json::Value;

use crate::with_ctx;

/// Creates a metadata statement and returns the ID of the statement and the CID of the metadata
/// Json
#[pyfunction]
#[pyo3(signature = (subject, metadata, *, timestamp=None, graph_id=None))]
pub fn create_metadata_statement(
    py: Python,
    subject: String,
    metadata: String,
    timestamp: Option<String>,
    graph_id: Option<uuid::Uuid>,
) -> PyResult<(String, String)> {
    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph_id);
        let metadata_json: Value =
            serde_json::from_str(&metadata).context("Invalid metadata JSON")?;

        let signer = ctx
            .active_signer
            .ok_or_else(|| anyhow!("No active signer available"))?;

        let metadata_statement = MetadataStatement::create_from_json(
            subject,
            metadata_json,
            signer.get_did_doc().id,
            timestamp.clone(),
        )
        .await?;

        let metadata = metadata_statement.metadata.clone();
        let statement = Statement::MetadataRegistration(metadata_statement);

        ctx.sql_lite
            .register_statement(&statement, &graph_id)
            .await?;

        Ok((statement.get_id(), metadata))
    })
}
