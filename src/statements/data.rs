use integrity::lineage::models::statements::{DataStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};
use uuid::Uuid;

use crate::{indexer::Graph, resolve_skip_proof, resolve_timestamp, with_ctx};

#[pyfunction]
#[pyo3(signature = (data, *, skip_proof=None, ctx=None))]
pub fn add_data_statement(
    py: Python,
    data: Vec<String>,
    skip_proof: Option<bool>,
    ctx: Option<Graph>,
) -> PyResult<Vec<String>> {
    let timestamp = resolve_timestamp(None);
    let graph_id = ctx.map(|graph| graph.id);

    let statement_id = create_data_statement(py, data, timestamp.clone(), graph_id)?;
    let mut statement_ids = vec![statement_id.clone()];

    if resolve_skip_proof(skip_proof) {
        log::info!("Skipping issuing of VC");
        return Ok(statement_ids);
    }

    let vc_id = super::vc::create_vc_statement(py, statement_id, timestamp, Some(Uuid::new_v4()))?;
    statement_ids.push(vc_id);

    Ok(statement_ids)
}

#[pyfunction]
#[pyo3(signature = (data, *, timestamp=None, graph_id=None))]
pub fn create_data_statement(
    py: Python,
    data: Vec<String>,
    timestamp: Option<String>,
    graph_id: Option<Uuid>,
) -> PyResult<String> {
    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph_id);
        let registered_by = ctx.clone().get_active_signer_did_key()?;

        let statement = Statement::DataRegistration(
            DataStatement::create(data, registered_by, timestamp).await?,
        );

        ctx.sql_lite
            .register_statement(&statement, &graph_id)
            .await?;

        Ok(statement.get_id())
    })
}
