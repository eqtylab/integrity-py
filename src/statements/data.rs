use integrity::lineage::models::statements::{DataStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};

use crate::context::{ctx, get_runtime};

#[pyfunction]
#[pyo3(signature = (data, *, timestamp=None, graph_id=None))]
pub fn create_data_statement(
    py: Python,
    data: Vec<String>,
    timestamp: Option<String>,
    graph_id: Option<uuid::Uuid>,
) -> PyResult<String> {
    // Get all context values before entering block_on to avoid blocking_read inside async context
    let context = ctx();
    let graph_id = context.resolve_graph_id(graph_id)?;
    let registered_by = context.clone().get_active_signer_did_key()?;
    let sql_lite = context.sql_lite.clone();

    py.detach(|| {
        get_runtime().block_on(async {
            let statement = Statement::DataRegistration(
                DataStatement::create(data, registered_by, timestamp).await?,
            );

            sql_lite.register_statement(&statement, &graph_id).await?;

            Ok(statement.get_id())
        })
    })
}
