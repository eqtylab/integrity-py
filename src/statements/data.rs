use integrity::lineage::models::statements::{DataStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};

use crate::context::{self, ctx};

#[pyfunction]
#[pyo3(signature = (data, *, timestamp=None, graph_id=None))]
pub fn create_data_statement(
    _py: Python,
    data: Vec<String>,
    timestamp: Option<String>,
    graph_id: Option<uuid::Uuid>,
) -> PyResult<String> {
    log::warn!("CREATING DATA STATEMENT");
    let graph_id = ctx().resolve_graph_id(graph_id)?;

    let registered_by = ctx().get_active_signer_did_key()?;

    let statement = Statement::DataRegistration(
        context::get_runtime().block_on(DataStatement::create(data, registered_by, timestamp))?,
    );

    context::get_runtime().block_on(ctx().sql_lite.register_statement(&statement, &graph_id))?;

    Ok(statement.get_id())
}
