use integrity::lineage::models::statements::{GovernanceStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    context::{self, ctx},
    to_py_err,
};

#[pyfunction]
#[pyo3(signature = (subject, document, *, timestamp=None, graph_id=None))]
pub fn create_governance_statement(
    _py: Python,
    subject: String,
    document: String,
    timestamp: Option<String>,
    graph_id: Option<String>,
) -> PyResult<String> {
    let graph_id = ctx().resolve_graph_id(graph_id).map_err(to_py_err)?;
    let registered_by = ctx().get_active_signer_did_key().map_err(to_py_err)?;

    let statement = Statement::GovernanceRegistration(
        context::get_runtime()
            .block_on(GovernanceStatement::create(
                subject,
                document,
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
