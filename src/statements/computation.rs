use integrity::{
    lineage::models::statements::{ComputationStatement, Statement, StatementTrait},
    signer::SignerType,
};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    context::{self, ctx},
    to_py_err,
};

#[pyfunction]
#[pyo3(signature = (inputs, outputs, *, computation=None, operated_by=None, executed_on=None, timestamp=None))]
pub fn create_computation_statement(
    _py: Python,
    inputs: Vec<String>,
    outputs: Vec<String>,
    computation: Option<String>,
    operated_by: Option<String>,
    executed_on: Option<String>,
    timestamp: Option<String>,
) -> PyResult<String> {
    let signer = ctx()
        .active_signer
        .ok_or_else(|| to_py_err("No active signer available"))?;
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
        context::get_runtime()
            .block_on(ComputationStatement::create(
                computation,
                inputs,
                outputs,
                operated_by,
                executed_on,
                registered_by,
                timestamp,
            ))
            .map_err(to_py_err)?,
    );

    context::get_runtime()
        .block_on(ctx().register_statement_locally(statement.clone(), None))
        .map_err(to_py_err)?;

    Ok(statement.get_id())
}
