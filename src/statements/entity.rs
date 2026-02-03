use integrity::lineage::models::statements::{EntityStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};

use crate::with_ctx;

#[pyfunction]
#[pyo3(signature = (entity, *, timestamp=None, graph_id=None))]
pub fn create_entity_statement(
    py: Python,
    entity: Vec<String>,
    timestamp: Option<String>,
    graph_id: Option<uuid::Uuid>,
) -> PyResult<String> {
    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph_id)?;
        let registered_by = ctx.clone().get_active_signer_did_key()?;

        let statement = Statement::EntityRegistration(
            EntityStatement::create(entity, registered_by, timestamp).await?,
        );

        ctx.sql_lite.register_statement(&statement, &graph_id).await?;

        Ok(statement.get_id())
    })
}
