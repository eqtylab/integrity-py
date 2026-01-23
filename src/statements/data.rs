use integrity::lineage::models::statements::{DataStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    context::{self, ctx},
    to_py_err,
};

#[pyfunction]
#[pyo3(signature = (data, *, timestamp=None))]
pub fn create_data_statement(
    _py: Python,
    data: Vec<String>,
    timestamp: Option<String>,
) -> PyResult<String> {
    let registered_by = ctx().get_active_signer_did_key().map_err(to_py_err)?;

    let statement = Statement::DataRegistration(
        context::get_runtime()
            .block_on(DataStatement::create(data, registered_by, timestamp))
            .map_err(to_py_err)?,
    );

    context::get_runtime()
        .block_on(ctx().register_statement_locally(statement.clone(), None))
        .map_err(to_py_err)?;

    Ok(statement.get_id())
}
