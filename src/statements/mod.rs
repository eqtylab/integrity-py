use integrity::lineage::models::statements::Statement;
use pyo3::prelude::*;

use crate::with_cfg;

mod association;
mod computation;
pub(crate) mod data;
pub(crate) mod did;
pub(crate) mod entity;
pub(crate) mod governance;
pub(crate) mod metadata;
pub(crate) mod model_signing;
/// Storage statement creation for referencing external data stores.
pub mod storage;
mod vc;
pub use association::PyAssociationType;

/// `statements` submodule to create lineage statements
#[pymodule]
pub fn statements(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<association::PyAssociationType>()?;
    m.add_function(wrap_pyfunction!(association::add_association_statement, m)?)?;
    m.add_function(wrap_pyfunction!(computation::add_computation_statement, m)?)?;
    m.add_function(wrap_pyfunction!(data::add_data_statement, m)?)?;
    m.add_function(wrap_pyfunction!(did::add_did_statement, m)?)?;
    m.add_function(wrap_pyfunction!(entity::add_entity_statement, m)?)?;
    m.add_function(wrap_pyfunction!(governance::add_governance_statement, m)?)?;
    m.add_function(wrap_pyfunction!(metadata::add_metadata_statement, m)?)?;
    m.add_function(wrap_pyfunction!(vc::add_vc_statement, m)?)?;

    m.add_function(wrap_pyfunction!(storage::add_storage_statement, m)?)?;
    m.add_function(wrap_pyfunction!(register_statement, m)?)?;

    m.add_function(wrap_pyfunction!(
        model_signing::create_model_signing_statement,
        m
    )?)?;

    Ok(())
}

/// Register a statement from JSON string to the default context.
#[pyfunction]
#[pyo3(signature = (statement_json))]
pub fn register_statement(py: Python, statement_json: String) -> PyResult<()> {
    with_cfg!(py, |ctx| {
        let statement: Statement = serde_json::from_str(&statement_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse statement JSON: {e}"))?;
        let context_id = ctx.default_context.id;
        ctx.sql_lite
            .register_statement(&statement, &context_id)
            .await?;
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}
