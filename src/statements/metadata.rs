use anyhow::{anyhow, Context as AnyhowContext};
use integrity::{
    blob_store::BlobStore,
    cid::multicodec,
    lineage::models::statements::{MetadataStatement, Statement, StatementTrait},
};
use pyo3::{pyfunction, PyResult, Python};
use serde_json::Value;

use crate::{
    config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_cfg, Graph, CID,
};

#[pyfunction]
#[pyo3(signature = (subject, metadata, *, skip_proof=None, graph=None))]
pub fn add_metadata_statement(
    py: Python,
    subject: String,
    metadata: String,
    skip_proof: Option<bool>,
    graph: Option<Graph>,
) -> PyResult<Vec<CID>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(skip_proof);

    with_cfg!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph);
        let metadata_json: Value =
            serde_json::from_str(&metadata).context("Invalid metadata JSON")?;

        let signer = ctx
            .clone()
            .active_signer
            .ok_or_else(|| anyhow!("No active signer available"))?;

        let metadata_statement = MetadataStatement::create_from_json(
            subject,
            metadata_json.clone(),
            signer.get_did_doc().id,
            timestamp.clone(),
        )
        .await?;

        let statement = Statement::MetadataRegistration(metadata_statement);

        ctx.sql_lite
            .register_statement(&statement, &graph_id)
            .await?;

        let id = CID::new(statement.get_id());
        let mut statement_ids = vec![id.clone()];

        log::debug!("Saving metadata json to blob store. {metadata_json}");
        ctx.blob_store
            .put(metadata_json.to_string().into(), multicodec::JSON_JCS, None)
            .await?;

        if !skip_proof {
            let vc_id = create_vc_for_statement(&ctx, &id, graph_id, timestamp).await?;
            statement_ids.push(vc_id);
        }

        Ok(statement_ids)
    })
}
