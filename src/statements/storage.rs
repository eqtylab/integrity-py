use integrity::lineage::models::statements::{Statement, StatementTrait, StorageStatement};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    context::{self, ctx},
    to_py_err,
};

/// Creates a storage statement.
#[pyfunction]
#[pyo3(signature = (data, stored_on, *, operated_by=None, timestamp=None, graph_id=None))]
pub fn create_storage_statement(
    _py: Python,
    data: String,
    stored_on: String,
    operated_by: Option<String>,
    timestamp: Option<String>,
    graph_id: Option<String>,
) -> PyResult<String> {
    let graph_id = ctx().resolve_graph_id(graph_id).map_err(to_py_err)?;
    let registered_by = ctx().get_active_signer_did_key().map_err(to_py_err)?;

    let statement = Statement::StorageRegistration(
        context::get_runtime()
            .block_on(StorageStatement::create(
                data,
                stored_on,
                operated_by,
                registered_by,
                timestamp,
            ))
            .map_err(to_py_err)?,
    );

    context::get_runtime()
        .block_on(ctx().sql_lite.register_statement(&statement, &graph_id))
        .map_err(to_py_err)?;

    Ok(statement.get_id())
}
