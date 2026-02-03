use integrity::lineage::models::statements::{AssociationStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};

use crate::context::{self, ctx};

#[pyfunction]
#[pyo3(signature = (subject, association, *, timestamp=None, graph_id=None))]
pub fn create_association_statement(
    _py: Python,
    subject: String,
    association: String,
    timestamp: Option<String>,
    graph_id: Option<uuid::Uuid>,
) -> PyResult<String> {
    let graph_id = ctx().resolve_graph_id(graph_id)?;
    let registered_by = ctx().get_active_signer_did_key()?;

    let statement = Statement::AssociationRegistration(context::get_runtime().block_on(
        AssociationStatement::create(subject, association, registered_by, timestamp),
    )?);

    context::get_runtime().block_on(ctx().sql_lite.register_statement(&statement, &graph_id))?;

    Ok(statement.get_id())
}
