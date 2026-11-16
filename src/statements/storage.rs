use integrity::lineage::models::statements::{Statement, StorageStatement};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    config::register_statement_with_vc, resolve_skip_proof, resolve_timestamp, with_cfg, Context,
    CID,
};

/// Adds a storage statement linking data to a storage location.
#[pyfunction]
#[pyo3(signature = (data, stored_on, *, operated_by=None, _skip_proof=None, context=None))]
pub fn add_storage_statement(
    py: Python,
    data: String,
    stored_on: String,
    operated_by: Option<String>,
    _skip_proof: Option<bool>,
    context: Option<Context>,
) -> PyResult<Vec<CID>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(_skip_proof);

    with_cfg!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(context);
        let registered_by = ctx.clone().get_active_signer_did_key()?;

        let statement = Statement::StorageRegistration(
            StorageStatement::create(
                data,
                stored_on,
                operated_by,
                registered_by,
                timestamp.clone(),
            )
            .await?,
        );

        Ok(register_statement_with_vc(&ctx, statement, graph_id, timestamp, skip_proof).await?)
    })
}
