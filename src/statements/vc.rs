use integrity::{
    lineage::models::statements::{Statement, StatementTrait, VcStatement},
    vc,
};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    context::{self, ctx},
    to_py_err,
};

#[pyfunction]
pub fn create_vc_statement(
    _py: Python,
    subject: String,
    timestamp: Option<String>,
) -> PyResult<String> {
    let signer = ctx()
        .active_signer
        .ok_or_else(|| to_py_err("No active signer available"))?;
    let registered_by = signer.get_did_doc().id;

    let vc = context::get_runtime()
        .block_on(vc::issue_vc(&subject, signer))
        .map_err(to_py_err)?;

    let statement = Statement::CredentialRegistration(
        context::get_runtime()
            .block_on(VcStatement::create(vc, registered_by, timestamp))
            .map_err(to_py_err)?,
    );

    context::get_runtime()
        .block_on(ctx().register_statement_locally(statement.clone(), None, None))
        .map_err(to_py_err)?;

    Ok(statement.get_id())
}
