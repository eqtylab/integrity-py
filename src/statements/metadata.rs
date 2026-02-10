use anyhow::{anyhow, Context as AnyhowContext};
use integrity::lineage::models::statements::{MetadataStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};
use serde_json::Value;
use uuid::Uuid;

use crate::{config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_ctx};

#[pyfunction]
#[pyo3(signature = (subject, metadata, *, skip_proof=None, graph_id=None))]
pub fn add_metadata_statement(
    py: Python,
    subject: String,
    metadata: String,
    skip_proof: Option<bool>,
    graph_id: Option<Uuid>,
) -> PyResult<Vec<String>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(skip_proof);

    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph_id);
        let metadata_json: Value =
            serde_json::from_str(&metadata).context("Invalid metadata JSON")?;

        let signer = ctx.clone()
            .active_signer
            .ok_or_else(|| anyhow!("No active signer available"))?;

        let metadata_statement = MetadataStatement::create_from_json(
            subject,
            metadata_json,
            signer.get_did_doc().id,
            timestamp.clone(),
        )
        .await?;

        let metadata = metadata_statement.metadata.clone();
        let statement = Statement::MetadataRegistration(metadata_statement);

        ctx.sql_lite
            .register_statement(&statement, &graph_id)
            .await?;

        let id = statement.get_id();
        let mut statement_ids = vec![id.clone()];

        let blob_dir = ctx.app_dir.join("blobs");
        let metadata_cid = metadata.trim_start_matches("urn:cid:");
        let metadata_file = blob_dir.join(metadata_cid);
        tokio::fs::write(metadata_file, metadata).await?;

        if !skip_proof {
            let vc_id = create_vc_for_statement(&ctx, &id, graph_id, timestamp).await?;
            statement_ids.push(vc_id);
        }

        Ok(statement_ids)
    })
}
