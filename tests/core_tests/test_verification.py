import json
import unittest
from pathlib import Path

from eqty_sdk import verify_statement, verify_vc

MANIFEST_PATH = Path(__file__).parents[2] / "src/indexer/testdata/simple.json"

# A known-good credential over a did:key issuer. Its `issuanceDate` routes it
# through the pre-ssi-0.16 verifier, so this also covers the legacy path.
VALID_VC = {
    "@context": ["https://www.w3.org/ns/credentials/v2", "https://w3id.org/security/v2"],
    "id": "urn:uuid:cf35933b-b49d-4b18-82ee-0e594912ec87",
    "type": ["VerifiableCredential"],
    "credentialSubject": {
        "id": "urn:cid:bafkr4ibthuzk3zug7ghmx63yjqaiu6rx4hhfdv3453j5bodskgw57bx2ya"
    },
    "issuer": "did:key:z6Mkt1QV8soXyenn4uUYtrMzFDnWWq8e8Mu71t2KmBsWi2mv",
    "issuanceDate": "2026-05-14T13:43:44Z",
    "proof": {
        "type": "Ed25519Signature2018",
        "proofPurpose": "assertionMethod",
        "verificationMethod": (
            "did:key:z6Mkt1QV8soXyenn4uUYtrMzFDnWWq8e8Mu71t2KmBsWi2mv"
            "#z6Mkt1QV8soXyenn4uUYtrMzFDnWWq8e8Mu71t2KmBsWi2mv"
        ),
        "created": "2026-05-14T13:43:44Z",
        "jws": (
            "eyJhbGciOiJFZERTQSIsImNyaXQiOlsiYjY0Il0sImI2NCI6ZmFsc2V9.."
            "P1CYP_-UNuPSyJUfE3EfLnHDZxHE1rZt961j1UQ6wx0f4ftTs3cUNmQ6pINp6VEC"
            "GscjWnmvYtt4r2jt1-0YDg"
        ),
    },
    "validFrom": "2026-05-14T13:43:44Z",
}

VC_SUBJECT_ID = VALID_VC["credentialSubject"]["id"]

# A context this build does not embed, so it can only be resolved when the
# caller supplies it.
CUSTOM_CONTEXT_URI = "https://example.invalid/custom-context"
CUSTOM_CONTEXT = {
    "@context": {
        "@version": 1.1,
        "note": "https://example.invalid/vocab#note",
    }
}
CUSTOM_CONTEXT_STATEMENT = {
    "@context": CUSTOM_CONTEXT_URI,
    "@id": "urn:cid:bagb6qaq6ea66rd2zfa42cd244f2fklrswplnn2cgjow5mlo3m5uggjl4symek",
    "note": "a statement written against a context outside the embedded set",
}


def load_statements():
    return list(json.loads(MANIFEST_PATH.read_text())["statements"].values())


class VerifyStatementTests(unittest.TestCase):
    """Verification must work on a bare import, so these deliberately never call setup_sdk()."""

    @classmethod
    def setUpClass(cls):
        cls.statements = load_statements()
        cls.statement = cls.statements[0]

    def test_valid_statement(self):
        self.assertTrue(verify_statement(json.dumps(self.statement)))

    def test_every_statement_in_the_manifest(self):
        # The shape a caller loops in, and what the explorer UI does in TypeScript.
        for statement in self.statements:
            with self.subTest(id=statement["@id"]):
                self.assertTrue(verify_statement(json.dumps(statement)))

    def test_tampered_statement(self):
        tampered = {**self.statement, "timestamp": "2026-07-13T00:00:00Z"}
        self.assertFalse(verify_statement(json.dumps(tampered)))

    def test_tampered_registered_by(self):
        tampered = {**self.statement, "registeredBy": "did:key:z6MkwrongDidValueHere"}
        self.assertFalse(verify_statement(json.dumps(tampered)))

    def test_malformed_json_raises(self):
        with self.assertRaises(ValueError):
            verify_statement("{not-json")

    def test_non_object_raises(self):
        with self.assertRaises(ValueError):
            verify_statement("[]")

    def test_missing_id_raises(self):
        without_id = {k: v for k, v in self.statement.items() if k != "@id"}
        with self.assertRaises(ValueError):
            verify_statement(json.dumps(without_id))

    def test_non_string_id_raises(self):
        with self.assertRaises(ValueError):
            verify_statement(json.dumps({**self.statement, "@id": 7}))

    def test_unresolvable_context_raises(self):
        unknown = {**self.statement, "@context": "https://example.invalid/not-embedded"}
        with self.assertRaises(RuntimeError):
            verify_statement(json.dumps(unknown))

    def test_supplied_context_makes_an_unknown_context_resolvable(self):
        # Same document twice: without the context it cannot be canonicalized at
        # all, with it the check completes and returns an answer. Reaching a
        # verdict is the point here, not which verdict.
        with self.assertRaises(RuntimeError):
            verify_statement(json.dumps(CUSTOM_CONTEXT_STATEMENT))

        self.assertIsInstance(
            verify_statement(
                json.dumps(CUSTOM_CONTEXT_STATEMENT), {CUSTOM_CONTEXT_URI: CUSTOM_CONTEXT}
            ),
            bool,
        )

    def test_context_accepted_as_dict_or_as_json_string(self):
        statement_json = json.dumps(CUSTOM_CONTEXT_STATEMENT)

        as_dict = verify_statement(statement_json, {CUSTOM_CONTEXT_URI: CUSTOM_CONTEXT})
        as_text = verify_statement(statement_json, {CUSTOM_CONTEXT_URI: json.dumps(CUSTOM_CONTEXT)})

        self.assertEqual(as_dict, as_text)

    def test_invalid_context_json_string_raises(self):
        with self.assertRaises(ValueError):
            verify_statement(json.dumps(self.statement), {"urn:cid:whatever": "{not-json"})


class VerifyVcTests(unittest.TestCase):
    def test_valid_proof(self):
        self.assertTrue(verify_vc(json.dumps(VALID_VC)))

    def test_valid_proof_with_matching_subject(self):
        self.assertTrue(verify_vc(json.dumps(VALID_VC), VC_SUBJECT_ID))

    def test_subject_mismatch_is_false(self):
        self.assertFalse(verify_vc(json.dumps(VALID_VC), "urn:cid:some-other-statement"))

    def test_tampered_proof_is_false(self):
        tampered = {**VALID_VC, "validFrom": "2026-05-15T13:43:44Z"}
        self.assertFalse(verify_vc(json.dumps(tampered)))

    def test_subject_as_single_element_array(self):
        wrapped = {**VALID_VC, "credentialSubject": [VALID_VC["credentialSubject"]]}
        self.assertTrue(verify_vc(json.dumps(wrapped), VC_SUBJECT_ID))

    def test_multiple_subjects_raises(self):
        multi = {
            **VALID_VC,
            "credentialSubject": [
                VALID_VC["credentialSubject"],
                {"id": "urn:cid:another"},
            ],
        }
        with self.assertRaises(ValueError):
            verify_vc(json.dumps(multi))

    def test_missing_subject_raises(self):
        without = {k: v for k, v in VALID_VC.items() if k != "credentialSubject"}
        with self.assertRaises(ValueError):
            verify_vc(json.dumps(without))

    def test_malformed_json_raises(self):
        with self.assertRaises(ValueError):
            verify_vc("{not-json")

    def test_network_backed_did_method_raises(self):
        # did:web would resolve over HTTPS. Refusing is what keeps a network
        # failure from being reported as an invalid signature.
        non_offline = {
            **VALID_VC,
            "issuer": "did:web:example.com",
            "proof": {**VALID_VC["proof"], "verificationMethod": "did:web:example.com#key-1"},
        }
        with self.assertRaises(RuntimeError):
            verify_vc(json.dumps(non_offline))


if __name__ == "__main__":
    unittest.main()
