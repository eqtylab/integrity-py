use integrity::lineage::models::statements::{MetadataStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};
use serde_json::Value;

use crate::{
    context::{self, ctx},
    to_py_err,
};

/// Creates a metadata statement and returns the ID of the statement and the CID of the metadata
/// Json
#[pyfunction]
pub fn create_metadata_statement(
    _py: Python,
    subject: String,
    metadata: String,
    timestamp: Option<String>,
) -> PyResult<(String, String)> {
    let metadata_json: Value = serde_json::from_str(&metadata).map_err(to_py_err)?;

    let signer = ctx()
        .active_signer
        .ok_or_else(|| to_py_err("No active signer available"))?;

    let metadata_statement = context::get_runtime()
        .block_on(MetadataStatement::create_from_json(
            subject,
            metadata_json,
            signer.get_did_doc().id,
            timestamp.clone(),
        ))
        .map_err(to_py_err)?;

    let metadata = metadata_statement.metadata.clone();
    let statement = Statement::MetadataRegistration(metadata_statement);

    context::get_runtime()
        .block_on(ctx().register_statement_locally(statement.clone(), None))
        .map_err(to_py_err)?;

    Ok((statement.get_id(), metadata))
}
