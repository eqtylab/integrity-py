use integrity::lineage::models::statements::{GovernanceStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};

use crate::context::{self, ctx};

#[pyfunction]
#[pyo3(signature = (subject, document, *, timestamp=None, graph_id=None))]
pub fn create_governance_statement(
    _py: Python,
    subject: String,
    document: String,
    timestamp: Option<String>,
    graph_id: Option<uuid::Uuid>,
) -> PyResult<String> {
    let graph_id = ctx().resolve_graph_id(graph_id)?;
    let registered_by = ctx().get_active_signer_did_key()?;

    let statement = Statement::GovernanceRegistration(context::get_runtime().block_on(
        GovernanceStatement::create(subject, document, registered_by, timestamp),
    )?);

    context::get_runtime().block_on(ctx().sql_lite.register_statement(&statement, &graph_id))?;

    Ok(statement.get_id())
}
