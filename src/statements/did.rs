use integrity::lineage::models::statements::{
    DidStatement, DidStatementRegular, Statement, StatementTrait,
};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_cfg, Context, CID,
};

#[pyfunction]
#[pyo3(signature = (did, *, skip_proof=None, graph=None))]
pub fn add_did_statement(
    py: Python,
    did: String,
    skip_proof: Option<bool>,
    graph: Option<Context>,
) -> PyResult<Vec<CID>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(skip_proof);

    with_cfg!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph);

        let registered_by = ctx.clone().get_active_signer_did_key()?;

        let statement = Statement::DidRegistration(Box::new(DidStatement::Regular(
            DidStatementRegular::create(did, registered_by, timestamp.clone()).await?,
        )));

        ctx.sql_lite
            .register_statement(&statement, &graph_id)
            .await?;

        let id: CID = statement.get_id().into();
        let mut statement_ids: Vec<CID> = vec![id.clone()];

        if !skip_proof {
            let vc_id = create_vc_for_statement(&ctx, &id, graph_id, timestamp).await?;
            statement_ids.push(vc_id);
        }

        Ok(statement_ids)
    })
}
