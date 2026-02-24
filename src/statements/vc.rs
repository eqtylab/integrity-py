use anyhow::anyhow;
use integrity::{
    lineage::models::statements::{Statement, StatementTrait, VcStatement},
    vc,
};
use pyo3::{pyfunction, PyResult, Python};

use crate::with_ctx;

#[pyfunction]
#[pyo3(signature = (subject, *, timestamp=None, graph_id=None))]
pub fn add_vc_statement(
    py: Python,
    subject: String,
    timestamp: Option<String>,
    graph_id: Option<uuid::Uuid>,
) -> PyResult<String> {
    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph_id);
        let signer = ctx
            .active_signer
            .ok_or_else(|| anyhow!("No active signer available"))?;
        let registered_by = signer.get_did_doc().id.clone();

        let vc = vc::issue_vc(&subject, signer).await?;

        let statement = Statement::CredentialRegistration(
            VcStatement::create(vc, registered_by, timestamp).await?,
        );

        ctx.sql_lite
            .register_statement(&statement, &graph_id)
            .await?;

        Ok(statement.get_id())
    })
}

#[cfg(test)]
mod tests {
    use integrity::{
        lineage::models::statements::{Statement, StatementTrait, VcStatement},
        signer::{Ed25519Signer, SignerType},
        vc,
    };
    use pyo3_async_runtimes::tokio::get_runtime;
    use ssi::vc::Credential;
    use tempfile::tempdir;

    use crate::config::{ctx_async, Config};

    /// Creates a minimal valid W3C VC for testing
    fn create_test_credential() -> Credential {
        let vc_json = serde_json::json!({
            "@context": "https://www.w3.org/2018/credentials/v1",
            "type": ["VerifiableCredential"],
            "id": "urn:uuid:12345678-1234-1234-1234-123456789012",
            "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            "issuanceDate": "2024-01-01T00:00:00Z",
            "credentialSubject": {
                "id": "urn:cid:bafkr4ibthuzk3zug7ghmx63yjqaiu6rx4hhfdv3453j5bodskgw57bx2ya"
            }
        });

        Credential::from_json_unsigned(&serde_json::to_string(&vc_json).unwrap()).unwrap()
    }

    #[test]
    fn test_create_test_credential() {
        let credential = create_test_credential();
        assert!(credential.id.is_some());
    }

    #[test]
    fn test_vc_statement_creation_from_credential() {
        let credential = create_test_credential();
        let registered_by = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();
        let timestamp = "2024-06-27T21:40:37Z".to_string();

        let statement = get_runtime()
            .block_on(VcStatement::create(
                credential,
                registered_by.clone(),
                Some(timestamp.clone()),
            ))
            .unwrap();

        assert_eq!(statement.type_, "CredentialRegistration");
        assert_eq!(statement.registered_by, registered_by);
        assert_eq!(statement.timestamp, timestamp);
        assert!(statement.get_id().starts_with("urn:cid:"));
    }

    #[test]
    fn test_vc_statement_deterministic_cid() {
        let credential1 = create_test_credential();
        let credential2 = create_test_credential();
        let registered_by = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();
        let timestamp = "2024-06-27T21:40:37Z".to_string();

        let statement1 = get_runtime()
            .block_on(VcStatement::create(
                credential1,
                registered_by.clone(),
                Some(timestamp.clone()),
            ))
            .unwrap();

        let statement2 = get_runtime()
            .block_on(VcStatement::create(
                credential2,
                registered_by,
                Some(timestamp),
            ))
            .unwrap();

        assert_eq!(statement1.get_id(), statement2.get_id());
    }

    #[test]
    fn test_vc_statement_wraps_in_statement_enum() {
        let credential = create_test_credential();
        let registered_by = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();

        let vc_statement = get_runtime()
            .block_on(VcStatement::create(credential, registered_by, None))
            .unwrap();

        let statement = Statement::CredentialRegistration(vc_statement.clone());

        assert_eq!(statement.get_id(), vc_statement.get_id());
    }

    #[test]
    fn test_issue_vc_with_ed25519_signer() {
        let signer = Ed25519Signer::create().unwrap();
        let signer_type = SignerType::ED25519(signer);
        let subject = "urn:cid:bafkr4ibthuzk3zug7ghmx63yjqaiu6rx4hhfdv3453j5bodskgw57bx2ya";

        let credential = get_runtime()
            .block_on(vc::issue_vc(subject, signer_type))
            .unwrap();

        assert!(credential.proof.is_some());
    }

    #[test]
    fn test_full_vc_statement_flow() {
        // Create a signer
        let signer = Ed25519Signer::create().unwrap();
        let signer_type = SignerType::ED25519(signer);
        let registered_by = signer_type.get_did_doc().id.clone();

        // Issue a VC
        let subject = "urn:cid:bafkr4ibthuzk3zug7ghmx63yjqaiu6rx4hhfdv3453j5bodskgw57bx2ya";
        let credential = get_runtime()
            .block_on(vc::issue_vc(subject, signer_type))
            .unwrap();

        // Create the statement
        let statement = get_runtime()
            .block_on(VcStatement::create(credential, registered_by.clone(), None))
            .unwrap();

        assert_eq!(statement.type_, "CredentialRegistration");
        assert_eq!(statement.registered_by, registered_by);
        assert!(statement.get_id().starts_with("urn:cid:"));
    }

    #[test]
    fn test_context_without_signer_returns_none() {
        let temp_dir = tempdir().unwrap();
        get_runtime().block_on(async {
            Config::reset_internal().await.unwrap();
            let ctx = Config::init(temp_dir.path().to_path_buf()).await.unwrap();
            assert!(ctx.active_signer.is_none());
        });
    }

    #[test]
    fn test_context_with_signer() {
        let temp_dir = tempdir().unwrap();
        get_runtime().block_on(async {
            Config::reset_internal().await.unwrap();
            let _ = Config::init(temp_dir.path().to_path_buf()).await.unwrap();

            // Create and set signer
            let signer = Ed25519Signer::create().unwrap();
            let signer_type = SignerType::ED25519(signer);
            let expected_did = signer_type.get_did_doc().id.clone();
            Config::set_active_signer_async(signer_type).await.unwrap();

            // Verify signer was set
            let ctx = ctx_async().await;
            assert!(ctx.active_signer.is_some());
            assert_eq!(ctx.active_signer.unwrap().get_did_doc().id, expected_did);
        });
    }
}
