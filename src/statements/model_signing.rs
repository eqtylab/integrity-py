use std::sync::Arc;

use anyhow::{anyhow, Context as AnyhowContext};
use integrity::{
    intoto_attestation::sign_intoto_attestation,
    lineage::models::statements::{SigstoreBundleStatement, Statement, StatementTrait},
    model_signing::{
        create_model_signing_intoto_statement, create_model_signing_sigstore_bundle, DirectoryInfo,
    },
};
use pyo3::{pyfunction, PyResult, Python};
use serde_json::Value;

use crate::{resolve_timestamp, with_cfg, Context, CID};

#[pyfunction]
#[pyo3(signature = (collection_cid, model_signing_name, *, context=None))]
pub fn add_model_signing_statement(
    py: Python,
    collection_cid: String,
    model_signing_name: String,
    // ignore_paths: Vec<String>,
    context: Option<Context>,
) -> PyResult<CID> {
    let timestamp = resolve_timestamp(None);
    let collection_cid = collection_cid.trim_start_matches("urn:cid:").to_string();
    with_cfg!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(context);

        let blob_store = Arc::new(ctx.blob_store);
        let allow_symlinks = ctx.cid_ignore.include_symlinks;

        let intoto_statement = create_model_signing_intoto_statement(
            model_signing_name,
            DirectoryInfo::IrohCollectionCidAndBlobStore(collection_cid.clone(), blob_store),
            allow_symlinks,
            vec![], // ignore_paths,
        )
        .await?;

        let signer = Arc::new(ctx.active_signer.clone().ok_or_else(|| {
            anyhow!("No active signer available to sign the model signing intoto statement")
        })?);

        let dsse =
            sign_intoto_attestation(intoto_statement, Arc::new(signer.signer.clone())).await?;
        let dsse = serde_json::from_str::<Value>(&dsse).context("Failed to parse DSSE")?;

        let sigstore_bundle =
            create_model_signing_sigstore_bundle(dsse, &signer.signer.get_did_doc().id)?;

        let sigstore_bundle_statement = {
            let subject = format!("urn:cid:{collection_cid}");
            let registered_by = signer.signer.get_did_doc().id;

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

        Ok(sigstore_bundle_statement.get_id().into())
    })
}
