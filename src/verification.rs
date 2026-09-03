use std::{collections::HashMap, fmt};

use anyhow::{anyhow, Result};
use integrity::{
    cid::{blake3::blake3_cid, multicodec},
    json_ld::to_nquads::jsonld_to_nquads,
    nquads::canonicalize_nquads,
    vc,
};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyAny};
use pyo3_async_runtimes::tokio::get_runtime;
use serde_json::Value;

/// DID methods whose document is derived from the identifier itself, and which
/// therefore resolve without network access.
///
/// `ssi`'s `AnyDidMethod` also handles `did:web` (HTTPS), `did:ethr` (JSON-RPC),
/// `did:ion` and `did:tz`, all of which make network requests.
const OFFLINE_DID_METHODS: [&str; 3] = ["did:key:", "did:jwk:", "did:pkh:"];

/// Why a verification could not be completed.
///
/// Verification answers `true` or `false`; this covers the cases where there is
/// no answer to give. The split decides which Python exception the caller sees,
/// so it is carried in the type rather than recovered from the message.
#[derive(Debug)]
enum VerifyError {
    /// Input that is not the kind of document we were asked to check. The
    /// caller can fix it, so this surfaces as `ValueError`.
    Malformed(String),
    /// Everything else: an unresolvable context, a DID method needing the
    /// network. Surfaces as `RuntimeError`, this crate's usual conversion.
    Failed(anyhow::Error),
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(message) => write!(f, "{message}"),
            Self::Failed(error) => write!(f, "{error:#}"),
        }
    }
}

impl From<anyhow::Error> for VerifyError {
    fn from(error: anyhow::Error) -> Self {
        Self::Failed(error)
    }
}

impl From<VerifyError> for PyErr {
    fn from(error: VerifyError) -> Self {
        match error {
            VerifyError::Malformed(message) => PyValueError::new_err(message),
            VerifyError::Failed(error) => PyErr::from(error),
        }
    }
}

/// Shorthand for the malformed-input case.
fn malformed<T>(message: impl Into<String>) -> Result<T, VerifyError> {
    Err(VerifyError::Malformed(message.into()))
}

/// Parses a document that must be a JSON object.
fn parse_object(json: &str, what: &str) -> Result<Value, VerifyError> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| VerifyError::Malformed(format!("{what} is not valid JSON: {e}")))?;
    if !value.is_object() {
        return malformed(format!("{what} is not a JSON object"));
    }
    Ok(value)
}

/// Converts a `{uri: context}` map into the `{uri: json_text}` form the JSON-LD
/// loader expects.
///
/// Each value may be a mapping or a JSON string, so a manifest's `contexts`
/// field can be passed through in either shape. The text must be the whole
/// context document, meaning the object containing the `@context` key.
fn contexts_to_json_text(
    py: Python<'_>,
    contexts: Option<HashMap<String, Py<PyAny>>>,
) -> PyResult<Option<HashMap<String, String>>> {
    let Some(contexts) = contexts else {
        return Ok(None);
    };

    let json = py.import("json")?;
    let dumps = json.getattr("dumps")?;

    let mut converted = HashMap::with_capacity(contexts.len());
    for (uri, value) in contexts {
        let bound = value.bind(py);
        let text = if let Ok(text) = bound.extract::<String>() {
            serde_json::from_str::<Value>(&text).map_err(|e| {
                PyValueError::new_err(format!("context {uri:?} is not valid JSON: {e}"))
            })?;
            text
        } else {
            dumps.call1((bound,))?.extract::<String>()?
        };
        converted.insert(uri, text);
    }

    Ok(Some(converted))
}

/// Recomputes a statement's RDFC CID and compares it with the `@id` it carries.
///
/// `@id` is removed before canonicalization: it is the output of this
/// computation, so it cannot also be an input.
async fn statement_id_matches(
    statement_json: &str,
    contexts: Option<HashMap<String, String>>,
) -> Result<bool, VerifyError> {
    let mut statement = parse_object(statement_json, "statement")?;
    let object = statement.as_object_mut().expect("checked by parse_object");

    let Some(id) = object.remove("@id") else {
        return malformed("statement has no '@id'");
    };
    let Some(id) = id.as_str().map(str::to_owned) else {
        return malformed("statement '@id' is not a string");
    };

    let nquads = jsonld_to_nquads(statement, contexts).await.map_err(|e| {
        anyhow!(
            "failed to canonicalize statement: {e:#}. A context that is not embedded in this \
             build must be supplied via contexts="
        )
    })?;
    let canon_nquads = canonicalize_nquads(nquads)?;
    let cid = blake3_cid(multicodec::RDFC_1_0, canon_nquads.as_bytes())?;

    Ok(format!("urn:cid:{cid}") == id)
}

/// Extracts the identifier of the subject a credential is about.
///
/// Accepts a single `credentialSubject` object or a one-element array. A
/// credential with several subjects has no single identifier to bind to.
fn credential_subject_id(vc: &Value) -> Result<String, VerifyError> {
    let Some(subject) = vc.get("credentialSubject") else {
        return malformed("VC has no 'credentialSubject'");
    };

    let subject = match subject {
        Value::Object(_) => subject,
        Value::Array(subjects) => match subjects.as_slice() {
            [single] => single,
            [] => return malformed("VC 'credentialSubject' array is empty"),
            _ => return malformed("VC has multiple subjects, this is unsupported"),
        },
        _ => return malformed("VC 'credentialSubject' must be an object or an array"),
    };

    match subject.get("id").and_then(Value::as_str) {
        Some(id) => Ok(id.to_owned()),
        None => malformed("VC credential subject has no string 'id'"),
    }
}

/// Rejects credentials whose verification would reach the network.
///
/// Checks the issuer DID and the `verificationMethod` of every proof. This is
/// what makes it sound to report a failed verification as `false`: with a
/// self-contained DID method, the only remaining reason to fail is that the
/// proof does not check out.
fn ensure_offline_verifiable(vc: &Value) -> Result<(), VerifyError> {
    let offline = |did: &str| OFFLINE_DID_METHODS.iter().any(|m| did.starts_with(m));
    let reject = |kind: &str, did: &str| {
        VerifyError::Failed(anyhow!(
            "offline verification needs a DID method resolvable without network access \
             (did:key, did:jwk or did:pkh); {kind} is {did:?}"
        ))
    };

    let issuer = vc
        .get("issuer")
        .and_then(|issuer| {
            issuer
                .as_str()
                .or_else(|| issuer.get("id").and_then(Value::as_str))
        })
        .ok_or_else(|| VerifyError::Malformed("VC has no string issuer DID".to_owned()))?;
    if !offline(issuer) {
        return Err(reject("issuer", issuer));
    }

    let proofs = match vc.get("proof") {
        Some(proof @ Value::Object(_)) => vec![proof],
        Some(Value::Array(proofs)) if !proofs.is_empty() => proofs.iter().collect(),
        Some(Value::Array(_)) => return malformed("VC 'proof' array is empty"),
        Some(_) => return malformed("VC 'proof' must be an object or an array"),
        None => return malformed("VC has no proof"),
    };

    for proof in proofs {
        let Some(method) = proof.get("verificationMethod").and_then(Value::as_str) else {
            return malformed("VC proof has no string 'verificationMethod'");
        };
        if !offline(method) {
            return Err(reject("verificationMethod", method));
        }
    }

    Ok(())
}

/// Verifies a credential's proof and, optionally, the subject it is bound to.
async fn vc_verifies(
    vc_json: &str,
    expected_subject_id: Option<&str>,
    contexts: Option<HashMap<String, String>>,
) -> Result<bool, VerifyError> {
    let vc = parse_object(vc_json, "VC")?;

    let subject_id = credential_subject_id(&vc)?;
    ensure_offline_verifiable(&vc)?;

    if let Some(expected) = expected_subject_id {
        if subject_id != expected {
            log::debug!("VC subject {subject_id:?} does not match expected {expected:?}");
            return Ok(false);
        }
    }

    match vc::verify_vc(vc_json, contexts).await {
        Ok(_) => Ok(true),
        Err(e) => {
            log::debug!("VC proof did not verify: {e:#}");
            Ok(false)
        }
    }
}

/// Verifies that a lineage statement's content still hashes to its `@id`.
///
/// Canonicalizes the statement as JSON-LD, recomputes its BLAKE3 RDFC CID and
/// compares it with the `@id` the statement carries. Returns `False` when they
/// differ, meaning the statement was modified after it was created.
///
/// `contexts` maps a JSON-LD context URI to its context document, for contexts
/// this build does not embed. Values may be dicts or JSON strings, so a
/// manifest's `contexts` field can be passed straight through. Note that a
/// supplied context takes precedence over an embedded one of the same URI.
///
/// Runs fully offline. A context that is neither embedded nor supplied raises
/// rather than being fetched.
///
/// A `True` result means every field the statement's `@context` defines is
/// unmodified. It does not mean the bytes are unmodified: the identifier
/// commits to the statement's canonicalized RDF, and JSON-LD expansion drops
/// keys the context does not define.
///
/// Raises `ValueError` if the statement is not a JSON object or has no string
/// `@id`, and `RuntimeError` if a context cannot be resolved.
#[pyfunction]
#[pyo3(signature = (statement_json, contexts=None))]
pub fn verify_statement(
    py: Python<'_>,
    statement_json: String,
    contexts: Option<HashMap<String, Py<PyAny>>>,
) -> PyResult<bool> {
    let contexts = contexts_to_json_text(py, contexts)?;

    py.detach(|| get_runtime().block_on(statement_id_matches(&statement_json, contexts)))
        .map_err(PyErr::from)
}

/// Verifies a W3C Verifiable Credential's proof offline.
///
/// When `statement_id` is given, also checks that the credential's
/// `credentialSubject.id` is that statement. A valid signature over some other
/// subject says nothing about the statement in hand.
///
/// Returns `False` when the credential does not verify or is bound to a
/// different subject. Raises `ValueError` when the input is not a credential,
/// meaning it is not JSON, has no `credentialSubject`, or has more than one
/// subject. Raises `RuntimeError` when the credential's DID method would need
/// network access: only `did:key`, `did:jwk` and `did:pkh` resolve offline.
///
/// Checks the cryptographic proof only. Revocation and suspension live in the
/// credential's status list, which is fetched over the network and is not
/// consulted here.
///
/// `contexts` maps a JSON-LD context URI to its context document, for contexts
/// this build does not embed. Values may be dicts or JSON strings, so a
/// manifest's `contexts` field can be passed straight through. Note that a
/// supplied context takes precedence over an embedded one of the same URI.
///
/// Runs fully offline. Verifying a credential re-expands it, so a context that
/// is neither embedded nor supplied is never fetched — it reports `False`,
/// alongside the other reasons a proof may not check out.
#[pyfunction]
#[pyo3(signature = (vc_json, statement_id=None, contexts=None))]
pub fn verify_vc(
    py: Python<'_>,
    vc_json: String,
    statement_id: Option<String>,
    contexts: Option<HashMap<String, Py<PyAny>>>,
) -> PyResult<bool> {
    let contexts = contexts_to_json_text(py, contexts)?;

    py.detach(|| get_runtime().block_on(vc_verifies(&vc_json, statement_id.as_deref(), contexts)))
        .map_err(PyErr::from)
}

#[cfg(test)]
mod tests {
    use integrity::{
        lineage::models::statements::{DataStatement, Statement},
        signer::{Ed25519Signer, SignerType},
        vc as core_vc,
    };
    use pyo3_async_runtimes::tokio::get_runtime;
    use serde_json::json;

    use super::*;

    const SUBJECT: &str = "urn:cid:bafkr4ibthuzk3zug7ghmx63yjqaiu6rx4hhfdv3453j5bodskgw57bx2ya";

    fn data_statement() -> Value {
        let statement = get_runtime()
            .block_on(DataStatement::create(
                vec![SUBJECT.to_owned()],
                "did:key:z6Mkw2PvzC9DHXiYQHMDRwyxCCV9n4EDc6vqqp1uyi9nrwsP".to_owned(),
                Some("2026-07-13T00:00:00Z".to_owned()),
            ))
            .unwrap();
        serde_json::to_value(Statement::DataRegistration(statement)).unwrap()
    }

    fn signed_vc() -> String {
        let signer = SignerType::ED25519(Ed25519Signer::create().unwrap());
        let signed = get_runtime()
            .block_on(core_vc::issue_vc(SUBJECT, signer))
            .unwrap();
        serde_json::to_string(&signed).unwrap()
    }

    fn statement_of(json: &str) -> VerifyError {
        get_runtime()
            .block_on(statement_id_matches(json, None))
            .unwrap_err()
    }

    fn verify(statement: &Value) -> Result<bool, VerifyError> {
        get_runtime().block_on(statement_id_matches(&statement.to_string(), None))
    }

    #[test]
    fn accepts_a_freshly_created_statement() {
        assert!(verify(&data_statement()).unwrap());
    }

    #[test]
    fn rejects_a_modified_statement() {
        let mut tampered = data_statement();
        tampered["timestamp"] = json!("2026-07-14T00:00:00Z");
        assert!(!verify(&tampered).unwrap());
    }

    #[test]
    fn undefined_keys_are_not_covered_by_the_identifier() {
        // The statement contexts are @protected with no @vocab, so JSON-LD
        // expansion drops terms they do not define and they never reach the
        // canonicalized RDF the CID is computed over. Verification therefore
        // says "every defined field is unmodified", not "these bytes are
        // unmodified". Pinned so the limitation stays deliberate.
        let mut extended = data_statement();
        extended["somethingTheContextDoesNotDefine"] = json!("ignored");
        assert!(verify(&extended).unwrap());
    }

    #[test]
    fn reports_structural_problems_rather_than_returning_false() {
        let mut without_id = data_statement();
        without_id.as_object_mut().unwrap().remove("@id");
        assert!(verify(&without_id).unwrap_err().to_string().contains("@id"));

        let mut numeric_id = data_statement();
        numeric_id["@id"] = json!(7);
        assert!(verify(&numeric_id)
            .unwrap_err()
            .to_string()
            .contains("not a string"));

        assert!(get_runtime()
            .block_on(statement_id_matches("{not-json", None))
            .unwrap_err()
            .to_string()
            .contains("not valid JSON"));

        assert!(get_runtime()
            .block_on(statement_id_matches("[]", None))
            .unwrap_err()
            .to_string()
            .contains("not a JSON object"));
    }

    #[test]
    fn an_unresolvable_context_is_an_error_not_a_mismatch() {
        let mut unknown = data_statement();
        unknown["@context"] = json!("https://example.invalid/not-embedded");
        let error = verify(&unknown).unwrap_err().to_string();
        assert!(error.contains("contexts="), "unexpected error: {error}");
    }

    #[test]
    fn classifies_what_the_caller_can_fix_separately_from_what_they_cannot() {
        // The Malformed/Failed split is what decides ValueError vs RuntimeError,
        // so assert on the variant rather than on message text.
        let malformed_cases = [
            statement_of("{not-json"),
            statement_of("[]"),
            {
                let mut s = data_statement();
                s.as_object_mut().unwrap().remove("@id");
                statement_of(&s.to_string())
            },
            {
                let mut s = data_statement();
                s["@id"] = json!(7);
                statement_of(&s.to_string())
            },
        ];
        for error in malformed_cases {
            assert!(
                matches!(error, VerifyError::Malformed(_)),
                "expected Malformed, got {error:?}"
            );
        }

        // An unresolvable context is not the caller handing us a bad document.
        let mut unknown = data_statement();
        unknown["@context"] = json!("https://example.invalid/not-embedded");
        assert!(matches!(
            statement_of(&unknown.to_string()),
            VerifyError::Failed(_)
        ));

        // Nor is a DID method that would need the network.
        let web = json!({
            "credentialSubject": {"id": SUBJECT},
            "issuer": "did:web:example.com",
            "proof": {"verificationMethod": "did:web:example.com#key-1"},
        });
        assert!(matches!(
            get_runtime()
                .block_on(vc_verifies(&web.to_string(), None, None))
                .unwrap_err(),
            VerifyError::Failed(_)
        ));

        // But a credential with no subject is.
        assert!(matches!(
            get_runtime()
                .block_on(vc_verifies(r#"{"issuer": "did:key:z6Mk"}"#, None, None))
                .unwrap_err(),
            VerifyError::Malformed(_)
        ));
    }

    #[test]
    fn a_supplied_context_resolves_and_round_trips() {
        // A context outside the embedded set: unusable unless the caller
        // supplies it, and once supplied the identifier it produces verifies.
        const URI: &str = "https://example.invalid/custom-context";
        let context = json!({
            "@context": { "@version": 1.1, "note": "https://example.invalid/vocab#note" }
        });
        let contexts = HashMap::from([(URI.to_owned(), serde_json::to_string(&context).unwrap())]);

        let mut statement = json!({ "@context": URI, "note": "hello" });

        // Compute the identifier the way statement creation does, then verify it.
        let nquads = get_runtime()
            .block_on(jsonld_to_nquads(statement.clone(), Some(contexts.clone())))
            .unwrap();
        let cid = blake3_cid(
            multicodec::RDFC_1_0,
            canonicalize_nquads(nquads).unwrap().as_bytes(),
        )
        .unwrap();
        statement["@id"] = json!(format!("urn:cid:{cid}"));

        let statement_json = statement.to_string();
        assert!(get_runtime()
            .block_on(statement_id_matches(
                &statement_json,
                Some(contexts.clone())
            ))
            .unwrap());

        // Without the context there is no verdict to give, only an error.
        assert!(get_runtime()
            .block_on(statement_id_matches(&statement_json, None))
            .is_err());

        // And the identifier still commits to the content.
        statement["note"] = json!("goodbye");
        assert!(!get_runtime()
            .block_on(statement_id_matches(&statement.to_string(), Some(contexts)))
            .unwrap());
    }

    #[test]
    fn verifies_a_credential_and_its_subject_binding() {
        let vc_json = signed_vc();
        get_runtime().block_on(async {
            assert!(vc_verifies(&vc_json, None, None).await.unwrap());
            assert!(vc_verifies(&vc_json, Some(SUBJECT), None).await.unwrap());
            assert!(!vc_verifies(&vc_json, Some("urn:cid:different"), None)
                .await
                .unwrap());
        });
    }

    /// A credential whose `@context` this build does not embed verifies only
    /// when the caller supplies the document. Nothing is fetched, so without it
    /// there is no vocabulary to expand the custom term against.
    #[test]
    fn a_supplied_context_makes_a_credential_verifiable() {
        const URI: &str = "https://example.invalid/custom-vc-context";

        let signer = SignerType::ED25519(Ed25519Signer::create().unwrap());
        let issuer = signer.get_did_doc().id;
        let contexts = HashMap::from([(
            URI.to_string(),
            json!({"@context": {"@version": 1.1, "note": "https://example.invalid/terms/note"}})
                .to_string(),
        )]);

        let unsigned: core_vc::Credential = serde_json::from_value(json!({
            // security/v2 defines the Data-Integrity proof terms; without it the
            // proof configuration itself fails to expand.
            "@context": ["https://www.w3.org/ns/credentials/v2", "https://w3id.org/security/v2", URI],
            "type": ["VerifiableCredential"],
            "id": "urn:uuid:6f1d3f8a-0d2b-4c1f-9a7e-2b5c8d4e1f30",
            "issuer": issuer,
            "validFrom": "2026-01-01T00:00:00Z",
            "credentialSubject": {"id": SUBJECT, "note": "hello"},
        }))
        .unwrap();

        let vc_json = get_runtime().block_on(async {
            let signed = core_vc::sign_vc(unsigned, signer, Some(contexts.clone()))
                .await
                .expect("signing needs the same context verification does");
            serde_json::to_string(&signed).unwrap()
        });

        get_runtime().block_on(async {
            assert!(vc_verifies(&vc_json, Some(SUBJECT), Some(contexts))
                .await
                .unwrap());
            // Reports `false` rather than raising: unlike the statement path, the
            // VC path folds an unresolvable context in with a bad proof.
            assert!(!vc_verifies(&vc_json, Some(SUBJECT), None).await.unwrap());
        });
    }

    #[test]
    fn rejects_a_tampered_credential() {
        let mut tampered: Value = serde_json::from_str(&signed_vc()).unwrap();
        tampered["credentialSubject"]["id"] = json!("urn:cid:tampered");
        let vc_json = tampered.to_string();
        // Subject no longer matches, and the proof no longer covers the content.
        assert!(!get_runtime()
            .block_on(vc_verifies(&vc_json, Some(SUBJECT), None))
            .unwrap());
        assert!(!get_runtime()
            .block_on(vc_verifies(&vc_json, None, None))
            .unwrap());
    }

    #[test]
    fn extracts_subject_ids_in_both_accepted_shapes() {
        let single = json!({"credentialSubject": {"id": SUBJECT}});
        let wrapped = json!({"credentialSubject": [{"id": SUBJECT}]});
        assert_eq!(credential_subject_id(&single).unwrap(), SUBJECT);
        assert_eq!(credential_subject_id(&wrapped).unwrap(), SUBJECT);

        let multiple = json!({"credentialSubject": [{"id": SUBJECT}, {"id": "urn:cid:other"}]});
        assert!(credential_subject_id(&multiple)
            .unwrap_err()
            .to_string()
            .contains("multiple subjects"));

        assert!(credential_subject_id(&json!({}))
            .unwrap_err()
            .to_string()
            .contains("credentialSubject"));
    }

    #[test]
    fn refuses_did_methods_that_would_need_the_network() {
        let offline = json!({
            "issuer": "did:key:z6Mkt1QV8soXyenn4uUYtrMzFDnWWq8e8Mu71t2KmBsWi2mv",
            "proof": {"verificationMethod": "did:key:z6Mkt1QV8soXyenn4uUYtrMzFDnWWq8e8Mu71t2KmBsWi2mv#k"},
        });
        assert!(ensure_offline_verifiable(&offline).is_ok());

        let web_issuer = json!({
            "issuer": "did:web:example.com",
            "proof": {"verificationMethod": "did:key:z6Mkt1QV8soXyenn4uUYtrMzFDnWWq8e8Mu71t2KmBsWi2mv#k"},
        });
        assert!(ensure_offline_verifiable(&web_issuer).is_err());

        let web_method = json!({
            "issuer": "did:key:z6Mkt1QV8soXyenn4uUYtrMzFDnWWq8e8Mu71t2KmBsWi2mv",
            "proof": {"verificationMethod": "did:web:example.com#key-1"},
        });
        assert!(ensure_offline_verifiable(&web_method).is_err());

        let no_proof =
            json!({"issuer": "did:key:z6Mkt1QV8soXyenn4uUYtrMzFDnWWq8e8Mu71t2KmBsWi2mv"});
        assert!(ensure_offline_verifiable(&no_proof).is_err());
    }

    #[test]
    fn did_jwk_and_did_pkh_are_accepted_offline() {
        for did in ["did:jwk:eyJhbGciOiJFZERTQSJ9", "did:pkh:eip155:1:0xabc"] {
            let vc = json!({
                "issuer": did,
                "proof": {"verificationMethod": format!("{did}#key-1")},
            });
            assert!(
                ensure_offline_verifiable(&vc).is_ok(),
                "{did} should resolve offline"
            );
        }
    }
}
