use integrity::lineage::models::statements::{DataStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_cfg, Context, CID,
};

#[pyfunction]
#[pyo3(signature = (data, *, skip_proof=None, graph=None))]
pub fn add_data_statement(
    py: Python,
    data: Vec<CID>,
    skip_proof: Option<bool>,
    graph: Option<Context>,
) -> PyResult<Vec<CID>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(skip_proof);

    with_cfg!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph);
        let registered_by = ctx.clone().get_active_signer_did_key()?;

        let data: Vec<String> = data.into_iter().map(|cid| cid.to_string()).collect();
        let statement = Statement::DataRegistration(
            DataStatement::create(data, registered_by, timestamp.clone()).await?,
        );

        ctx.sql_lite
            .register_statement(&statement, &graph_id)
            .await?;

        let id = CID::new(statement.get_id());
        let mut statement_ids: Vec<CID> = vec![id.clone()];

        if !skip_proof {
            let vc_id = create_vc_for_statement(&ctx, &id, graph_id, timestamp).await?;
            statement_ids.push(vc_id);
        };

        Ok(statement_ids)
    })
}
