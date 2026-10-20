use integrity::lineage::models::statements::{
    compute_cid, Statement, StatementTrait, StorageStatement,
};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_cfg, Context, CID,
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

        let mut storage = StorageStatement::create(
            data.clone(),
            stored_on.clone(),
            operated_by,
            registered_by,
            timestamp.clone(),
        )
        .await?;

        // `StorageStatement::create` prepends `urn:cid:` to any value without it, turning
        // `urn:uuid:…` into `urn:cid:urn:uuid:…`. Keep URNs and DIDs as given, like
        // association and metadata statements, and recompute the ID to match.
        if is_urn_or_did(&data) || is_urn_or_did(&stored_on) {
            if is_urn_or_did(&data) {
                storage.data = data;
            }
            if is_urn_or_did(&stored_on) {
                storage.stored_on = stored_on;
            }
            let mut json = serde_json::to_value(&storage).map_err(anyhow::Error::from)?;
            json["@id"] = compute_cid(&storage).await?.into();
            storage = serde_json::from_value(json).map_err(anyhow::Error::from)?;
        }

        let statement = Statement::StorageRegistration(storage);

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

fn is_urn_or_did(value: &str) -> bool {
    value.starts_with("urn:") || value.starts_with("did:")
}
