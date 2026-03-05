use integrity::lineage::models::statements::{Statement, StatementTrait, StorageStatement};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_cfg, Context, CID,
};

/// Adds a storage statement linking data to a storage location.
#[pyfunction]
#[pyo3(signature = (data, stored_on, *, operated_by=None, skip_proof=None, context=None))]
pub fn add_storage_statement(
    py: Python,
    data: String,
    stored_on: String,
    operated_by: Option<String>,
    skip_proof: Option<bool>,
    context: Option<Context>,
) -> PyResult<Vec<CID>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(skip_proof);

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
