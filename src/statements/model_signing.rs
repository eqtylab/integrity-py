use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context as AnyhowContext};
use integrity::{
    blob_store,
    lineage::models::statements::{SigstoreBundleStatement, Statement, StatementTrait},
    model_signing::DirectoryInfo,
};
use pyo3::{pyfunction, PyResult, Python};
use serde_json::Value;

use crate::{with_ctx, Graph};

#[pyfunction]
#[pyo3(signature = (collection_cid, blobs_dir, model_signing_name, allow_symlinks, ignore_paths, *, timestamp=None, graph=None))]
pub fn create_model_signing_statement(
    py: Python,
    collection_cid: String,
    blobs_dir: PathBuf,
    model_signing_name: String,
    allow_symlinks: bool,
    ignore_paths: Vec<String>,
    timestamp: Option<String>,
    graph: Option<Graph>,
) -> PyResult<String> {
    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph);

        let blob_store = Arc::new(blob_store::local_fs::LocalFs::new(blobs_dir));

        let intoto_statement = integrity::model_signing::create_model_signing_intoto_statement(
            model_signing_name,
            DirectoryInfo::IrohCollectionCidAndBlobStore(collection_cid.clone(), blob_store),
            allow_symlinks,
            ignore_paths,
        )
        .await?;

        let signer = Arc::new(ctx.active_signer.clone().ok_or_else(|| {
            anyhow!("No active signer available to sign the model signing intoto statement")
        })?);

        let dsse = integrity::intoto_attestation::sign_intoto_attestation(
            intoto_statement,
            signer.clone(),
        )
        .await?;
        let dsse = serde_json::from_str::<Value>(&dsse).context("Failed to parse DSSE")?;

        let sigstore_bundle = integrity::model_signing::create_model_signing_sigstore_bundle(
            dsse,
            &signer.get_did_doc().id,
        )?;

        let sigstore_bundle_statement = {
            let subject = format!("urn:cid:{collection_cid}");
            let registered_by = signer.get_did_doc().id;

            Statement::CredentialSigstoreBundleRegistration(
                SigstoreBundleStatement::create(
                    subject,
                    &sigstore_bundle,
                    registered_by,
                    timestamp,
                )
                .await?,
            )
        };

        ctx.sql_lite
            .register_statement(&sigstore_bundle_statement, &graph_id)
            .await?;

        Ok(sigstore_bundle_statement.get_id())
    })
}
