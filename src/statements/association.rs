use integrity::lineage::models::statements::{AssociationStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};

use crate::with_ctx;

#[pyfunction]
#[pyo3(signature = (subject, association, *, timestamp=None, graph_id=None))]
pub fn create_association_statement(
    py: Python,
    subject: String,
    association: String,
    timestamp: Option<String>,
    graph_id: Option<uuid::Uuid>,
) -> PyResult<String> {
    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph_id);
        let registered_by = ctx.clone().get_active_signer_did_key()?;

        let statement = Statement::AssociationRegistration(
            AssociationStatement::create(subject, association, registered_by, timestamp).await?,
        );

        ctx.sql_lite
            .register_statement(&statement, &graph_id)
            .await?;

        Ok(statement.get_id())
    })
}
