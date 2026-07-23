use anyhow::{ensure, Context as AnyhowContext, Result};
use integrity::lineage::models::statements::Statement;
use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3_async_runtimes::tokio::get_runtime;
use serde_json::Value;

fn require_offline_did_key(value: &Value) -> Result<()> {
    let issuer = value
        .get("issuer")
        .and_then(|issuer| {
            issuer
                .as_str()
                .or_else(|| issuer.get("id").and_then(Value::as_str))
        })
        .context("invalid VC: missing string issuer DID")?;
    ensure!(
        issuer.starts_with("did:key:"),
        "offline VC verification requires a did:key issuer"
    );

    let proofs = match value.get("proof") {
        Some(Value::Object(proof)) => vec![proof],
        Some(Value::Array(proofs)) => proofs
            .iter()
            .map(|proof| {
                proof
                    .as_object()
                    .context("invalid VC: proof entries must be JSON objects")
            })
            .collect::<Result<Vec<_>>>()?,
        Some(_) => anyhow::bail!("invalid VC: proof must be a JSON object or array"),
        None => anyhow::bail!("invalid VC: missing Data Integrity proof"),
    };
    ensure!(
        !proofs.is_empty(),
        "invalid VC: proof array must not be empty"
    );

    for proof in proofs {
        let verification_method = proof
            .get("verificationMethod")
            .and_then(Value::as_str)
            .context("invalid VC: proof is missing string verificationMethod")?;
        ensure!(
            verification_method.starts_with("did:key:"),
            "offline VC verification requires did:key verification methods"
        );
    }

    Ok(())
}

async fn verify_vc_json(vc_json: &str) -> Result<()> {
    let value: Value =
        serde_json::from_str(vc_json).context("malformed VC JSON: expected a JSON object")?;
    ensure!(
        value.is_object(),
        "malformed VC JSON: expected a JSON object"
    );
    require_offline_did_key(&value)?;

    integrity::vc::verify_vc(vc_json)
        .await
        .context("VC verification failed")?;
    Ok(())
}

async fn statement_rdfc_cid_matches(statement_json: &str) -> Result<bool> {
    let mut value: Value = serde_json::from_str(statement_json)
        .context("malformed lineage statement JSON: expected a JSON object")?;
    ensure!(
        value.is_object(),
        "malformed lineage statement JSON: expected a JSON object"
    );

    let embedded_id = value
        .get("@id")
        .and_then(Value::as_str)
        .context("invalid lineage statement: missing string @id")?
        .to_owned();
    serde_json::from_value::<Statement>(value.clone())
        .context("invalid lineage statement: unsupported or incomplete statement")?;
    value
        .as_object_mut()
        .expect("lineage statement object was checked above")
        .remove("@id");
    let (recomputed_cid, _) = integrity::json_ld::compute_rdfc_cid_for_jsonld(value)
        .await
        .context("failed to recompute lineage statement RDFC CID")?;
    let recomputed_id = format!("urn:cid:{recomputed_cid}");

    Ok(recomputed_id == embedded_id)
}

/// Verifies a W3C Verifiable Credential's Data Integrity proof offline.
///
/// The credential must use a `did:key` verification method supported by the
/// pinned Integrity verifier. This checks the cryptographic proof only; it
/// does not perform network-backed credential-status or revocation checks.
/// Returns `True` when the proof is valid and raises `ValueError` otherwise.
#[pyfunction]
#[pyo3(signature = (vc_json))]
pub fn verify_vc(py: Python<'_>, vc_json: String) -> PyResult<bool> {
    py.detach(|| get_runtime().block_on(verify_vc_json(&vc_json)))
        .map(|_| true)
        .map_err(|error| PyValueError::new_err(error.to_string()))
}

/// Recomputes a lineage statement's RDFC CID and compares it with `@id`.
///
/// Returns `True` only when the canonicalized statement content produces the
/// exact CID embedded in `@id`. A well-formed but modified statement returns
/// `False`; malformed or unsupported statements raise `ValueError`.
#[pyfunction]
#[pyo3(signature = (statement_json))]
pub fn verify_statement_rdfc_cid(py: Python<'_>, statement_json: String) -> PyResult<bool> {
    py.detach(|| get_runtime().block_on(statement_rdfc_cid_matches(&statement_json)))
        .map_err(|error| PyValueError::new_err(error.to_string()))
}

#[cfg(test)]
mod tests {
    use integrity::{
        lineage::models::statements::{DataStatement, Statement},
        signer::{Ed25519Signer, SignerType},
        vc,
    };
    use pyo3_async_runtimes::tokio::get_runtime;
    use serde_json::{json, Value};

    use super::{statement_rdfc_cid_matches, verify_vc_json};

    fn signed_vc_json() -> String {
        let signer = SignerType::ED25519(Ed25519Signer::create().unwrap());
        let signed = get_runtime()
            .block_on(vc::issue_vc(
                "urn:cid:bafkr4ibthuzk3zug7ghmx63yjqaiu6rx4hhfdv3453j5bodskgw57bx2ya",
                signer,
            ))
            .unwrap();
        serde_json::to_string(&signed).unwrap()
    }

    fn data_statement_json() -> String {
        let statement = get_runtime()
            .block_on(DataStatement::create(
                vec![
                    "urn:cid:bafkr4ibthuzk3zug7ghmx63yjqaiu6rx4hhfdv3453j5bodskgw57bx2ya"
                        .to_owned(),
                ],
                "did:key:z6Mkw2PvzC9DHXiYQHMDRwyxCCV9n4EDc6vqqp1uyi9nrwsP".to_owned(),
                Some("2026-07-13T00:00:00Z".to_owned()),
            ))
            .unwrap();
        serde_json::to_string(&Statement::DataRegistration(statement)).unwrap()
    }

    #[test]
    fn accepts_valid_vc_and_rejects_tampering() {
        let vc_json = signed_vc_json();
        get_runtime().block_on(async {
            verify_vc_json(&vc_json).await.unwrap();

            let mut tampered: Value = serde_json::from_str(&vc_json).unwrap();
            tampered["credentialSubject"]["id"] = json!("urn:cid:tampered");
            let error = verify_vc_json(&serde_json::to_string(&tampered).unwrap())
                .await
                .unwrap_err();
            assert!(error.to_string().contains("VC verification failed"));
        });
    }

    #[test]
    fn reports_malformed_vc_json() {
        let error = get_runtime()
            .block_on(verify_vc_json("{not-json"))
            .unwrap_err();
        assert!(error.to_string().contains("malformed VC JSON"));
    }

    #[test]
    fn rejects_non_did_key_credentials() {
        let vc_json = json!({
            "issuer": "did:web:example.com",
            "proof": {
                "verificationMethod": "did:web:example.com#key-1"
            }
        })
        .to_string();

        let error = get_runtime()
            .block_on(verify_vc_json(&vc_json))
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("offline VC verification requires a did:key issuer"));
    }

    #[test]
    fn accepts_matching_statement_cid_and_detects_tampering() {
        let statement_json = data_statement_json();
        get_runtime().block_on(async {
            assert!(statement_rdfc_cid_matches(&statement_json).await.unwrap());

            let mut tampered: Value = serde_json::from_str(&statement_json).unwrap();
            tampered["timestamp"] = json!("2026-07-14T00:00:00Z");
            assert!(
                !statement_rdfc_cid_matches(&serde_json::to_string(&tampered).unwrap())
                    .await
                    .unwrap()
            );
        });
    }

    #[test]
    fn reports_missing_statement_id() {
        let error = get_runtime()
            .block_on(statement_rdfc_cid_matches(
                r#"{"@context":"urn:cid:context","@type":"DataRegistration"}"#,
            ))
            .unwrap_err();
        assert!(error.to_string().contains("missing string @id"));
    }
}
