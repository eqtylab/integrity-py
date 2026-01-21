use integrity::lineage::models::statements::{Statement, StatementTrait, StorageStatement};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    context::{self, ctx},
    to_py_err,
};

/// Creates a storage statement.
#[pyfunction]
pub fn create_storage_statement(
    _py: Python,
    data: String,
    stored_on: String,
    operated_by: Option<String>,
    timestamp: Option<String>,
) -> PyResult<String> {
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
        .block_on(ctx().register_statement_locally(statement.clone(), None, None))
        .map_err(to_py_err)?;

    Ok(statement.get_id())
}
