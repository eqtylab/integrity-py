use anyhow::anyhow;
use integrity::{
    lineage::models::statements::{ComputationStatement, Statement, StatementTrait},
    signer::SignerType,
};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_cfg, Graph, CID,
};

#[pyfunction]
#[pyo3(signature = (inputs, outputs, computation=None, *, operated_by=None, executed_on=None, skip_proof=None, graph=None))]
pub fn add_computation_statement(
    py: Python,
    inputs: Vec<CID>,
    outputs: Vec<CID>,
    computation: Option<CID>,
    operated_by: Option<String>,
    executed_on: Option<String>,
    skip_proof: Option<bool>,
    graph: Option<Graph>,
) -> PyResult<Vec<CID>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(skip_proof);

    with_cfg!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph);
        let signer = ctx
            .clone()
            .active_signer
            .ok_or_else(|| anyhow!("No active signer available"))?;

        // If VComp notary is being used, we fetch `operatedBy` and `executedOn`` from the signer
        let (operated_by, executed_on) = match &signer {
            SignerType::VCompNotarySigner(signer) => {
                let operated_by = if operated_by.is_some() {
                    operated_by
                } else {
                    signer.operated_by.clone()
                };
                let executed_on = if executed_on.is_some() {
                    executed_on
                } else {
                    signer.executed_on.clone()
                };

                (operated_by, executed_on)
            }
            _ => (operated_by, executed_on),
        };

        let registered_by = signer.get_did_doc().id;
        let operated_by = match operated_by {
            Some(operated_by) => operated_by,
            None => registered_by.clone(),
        };

        let computation = computation.map(|cid| cid.to_string());
        let inputs: Vec<String> = inputs.into_iter().map(|cid| cid.to_string()).collect();
        let outputs: Vec<String> = outputs.into_iter().map(|cid| cid.to_string()).collect();

        let statement = Statement::ComputationRegistration(
            ComputationStatement::create(
                computation,
                inputs,
                outputs,
                operated_by,
                executed_on,
                registered_by,
                timestamp.clone(),
            )
            .await?,
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
