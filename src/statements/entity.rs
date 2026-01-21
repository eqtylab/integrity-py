use integrity::lineage::models::statements::{EntityStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    context::{self, ctx},
    to_py_err,
};

#[pyfunction]
pub fn create_entity_statement(
    _py: Python,
    entity: Vec<String>,
    timestamp: Option<String>,
) -> PyResult<String> {
    let registered_by = ctx().get_active_signer_did_key().map_err(to_py_err)?;

    let statement = Statement::EntityRegistration(
        context::get_runtime()
            .block_on(EntityStatement::create(entity, registered_by, timestamp))
            .map_err(to_py_err)?,
    );

    context::get_runtime()
        .block_on(ctx().register_statement_locally(statement.clone(), None, None))
        .map_err(to_py_err)?;

    Ok(statement.get_id())
}
