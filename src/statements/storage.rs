use integrity::lineage::models::statements::{Statement, StatementTrait, StorageStatement};
use pyo3::{pyfunction, PyResult, Python};

use crate::with_ctx;

/// Creates a storage statement.
#[pyfunction]
#[pyo3(signature = (data, stored_on, *, operated_by=None, timestamp=None, graph_id=None))]
pub fn create_storage_statement(
    py: Python,
    data: String,
    stored_on: String,
    operated_by: Option<String>,
    timestamp: Option<String>,
    graph_id: Option<uuid::Uuid>,
) -> PyResult<String> {
    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph_id)?;
        let registered_by = ctx.clone().get_active_signer_did_key()?;

        let statement = Statement::StorageRegistration(
            StorageStatement::create(data, stored_on, operated_by, registered_by, timestamp).await?,
        );

        ctx.sql_lite.register_statement(&statement, &graph_id).await?;

        Ok(statement.get_id())
    })
}
