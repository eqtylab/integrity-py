use anyhow::anyhow;
use integrity::{
    lineage::models::statements::{ComputationStatement, Statement, StatementTrait},
    signer::SignerType,
};
use pyo3::{pyfunction, PyResult, Python};
use uuid::Uuid;

use crate::{config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_ctx};

#[pyfunction]
#[pyo3(signature = (inputs, outputs, computation=None, *, operated_by=None, executed_on=None, skip_proof=None, graph_id=None))]
pub fn add_computation_statement(
    py: Python,
    inputs: Vec<String>,
    outputs: Vec<String>,
    computation: Option<String>,
    operated_by: Option<String>,
    executed_on: Option<String>,
    skip_proof: Option<bool>,
    graph_id: Option<Uuid>,
) -> PyResult<Vec<String>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(skip_proof);

    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph_id);
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

        let id = statement.get_id();
        let mut statement_ids = vec![id.clone()];

        if !skip_proof {
            let vc_id = create_vc_for_statement(&ctx, &id, graph_id, timestamp).await?;
            statement_ids.push(vc_id);
        };

        Ok(statement_ids)
    })
}
