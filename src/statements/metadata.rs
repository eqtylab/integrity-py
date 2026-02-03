use integrity::lineage::models::statements::{MetadataStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};
use serde_json::Value;

use crate::context::{self, ctx};
use anyhow::{anyhow, Context as AnyhowContext};

/// Creates a metadata statement and returns the ID of the statement and the CID of the metadata
/// Json
#[pyfunction]
#[pyo3(signature = (subject, metadata, *, timestamp=None, graph_id=None))]
pub fn create_metadata_statement(
    _py: Python,
    subject: String,
    metadata: String,
    timestamp: Option<String>,
    graph_id: Option<uuid::Uuid>,
) -> PyResult<(String, String)> {
    let graph_id = ctx().resolve_graph_id(graph_id)?;
    let metadata_json: Value = serde_json::from_str(&metadata).context("Invalid metadata JSON")?;

    let signer = ctx()
        .active_signer
        .ok_or_else(|| anyhow!("No active signer available"))?;

    let metadata_statement =
        context::get_runtime().block_on(MetadataStatement::create_from_json(
            subject,
            metadata_json,
            signer.get_did_doc().id,
            timestamp.clone(),
        ))?;

    let metadata = metadata_statement.metadata.clone();
    let statement = Statement::MetadataRegistration(metadata_statement);

    context::get_runtime().block_on(ctx().sql_lite.register_statement(&statement, &graph_id))?;

    Ok((statement.get_id(), metadata))
}
