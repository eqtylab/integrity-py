use integrity::lineage::models::statements::{DidStatement, DidStatementRegular, Statement};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    config::register_statement_with_vc, resolve_skip_proof, resolve_timestamp, with_cfg, Context,
    CID,
};

#[pyfunction]
#[pyo3(signature = (did, *, _skip_proof=None, context=None))]
pub fn add_did_statement(
    py: Python,
    did: String,
    _skip_proof: Option<bool>,
    context: Option<Context>,
) -> PyResult<Vec<CID>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(_skip_proof);

    with_cfg!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(context);

        let registered_by = ctx.clone().get_active_signer_did_key()?;

        let statement = Statement::DidRegistration(Box::new(DidStatement::Regular(
            DidStatementRegular::create(did, registered_by, timestamp.clone()).await?,
        )));

        Ok(register_statement_with_vc(&ctx, statement, graph_id, timestamp, skip_proof).await?)
    })
}
