use anyhow::anyhow;
use integrity::{
    lineage::models::statements::{
        otel_span_urn as span_urn, ComputationSpan, ComputationStatement, Statement, StatementTrait,
    },
    signer::SignerType,
};
use pyo3::{exceptions::PyValueError, pyfunction, PyResult, Python};

use crate::{
    config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_cfg, Context, CID,
};

#[pyfunction]
#[pyo3(signature = (inputs, outputs, computation=None, *, operated_by=None, executed_on=None, start_time=None, end_time=None, span=None, parent_span=None, _skip_proof=None, context=None))]
#[allow(clippy::too_many_arguments)]
pub fn add_computation_statement(
    py: Python,
    inputs: Vec<CID>,
    outputs: Vec<CID>,
    computation: Option<CID>,
    operated_by: Option<String>,
    executed_on: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    span: Option<String>,
    parent_span: Option<String>,
    _skip_proof: Option<bool>,
    context: Option<Context>,
) -> PyResult<Vec<CID>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(_skip_proof);

    with_cfg!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(context);
        let signer = ctx
            .clone()
            .active_signer
            .ok_or_else(|| anyhow!("No active signer available"))?;

        // If VComp notary is being used, we fetch `operatedBy` and `executedOn`` from the signer
        let (operated_by, executed_on) = match &signer.signer {
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

        let registered_by = signer.signer.get_did_doc().id;
        let operated_by = match operated_by {
            Some(operated_by) => operated_by,
            None => registered_by.clone(),
        };

        let computation = computation.map(|cid| cid.to_string());
        let inputs: Vec<String> = inputs.into_iter().map(|cid| cid.to_string()).collect();
        let outputs: Vec<String> = outputs.into_iter().map(|cid| cid.to_string()).collect();
        let span = ComputationSpan {
            start_time,
            end_time,
            span,
            parent_span,
        };

        let statement = Statement::ComputationRegistration(
            ComputationStatement::create_with_span(
                computation,
                inputs,
                outputs,
                operated_by,
                executed_on,
                registered_by,
                timestamp.clone(),
                span,
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

/// The `urn:uuid` that identifies an OpenTelemetry span: a version 5 UUID of
/// `"{trace_id}/{span_id}"`, so a parent's is known from the `traceparent` alone.
#[pyfunction]
pub fn otel_span_urn(trace_id: &str, span_id: &str) -> PyResult<String> {
    span_urn(trace_id, span_id).map_err(|e| PyValueError::new_err(e.to_string()))
}
