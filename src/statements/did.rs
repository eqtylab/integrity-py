use integrity::lineage::models::statements::{
    DidStatement, DidStatementRegular, Statement, StatementTrait,
};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    context::{self, ctx},
    to_py_err,
};

#[pyfunction]
pub fn create_did_statement(
    _py: Python,
    did: String,
    timestamp: Option<String>,
) -> PyResult<String> {
    let registered_by = ctx().get_active_signer_did_key().map_err(to_py_err)?;

    let statement = Statement::DidRegistration(Box::new(DidStatement::Regular(
        context::get_runtime()
            .block_on(DidStatementRegular::create(did, registered_by, timestamp))
            .map_err(to_py_err)?,
    )));

    context::get_runtime()
        .block_on(ctx().register_statement_locally(statement.clone(), None, None))
        .map_err(to_py_err)?;

    Ok(statement.get_id())
}
