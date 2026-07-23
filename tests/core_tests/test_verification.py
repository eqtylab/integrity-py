import json
import unittest
from pathlib import Path

from eqty_sdk import verify_statement_rdfc_cid, verify_vc

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


class VerificationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        manifest_path = Path(__file__).parents[2] / "src/indexer/testdata/simple.json"
        manifest = json.loads(manifest_path.read_text())
        cls.statement = next(iter(manifest["statements"].values()))

    def test_valid_vc_proof(self):
        self.assertTrue(verify_vc(json.dumps(VALID_VC)))

    def test_tampered_vc_proof(self):
        tampered = {**VALID_VC, "validFrom": "2026-05-15T13:43:44Z"}

        with self.assertRaisesRegex(ValueError, "VC verification failed"):
            verify_vc(json.dumps(tampered))

    def test_malformed_vc_json(self):
        with self.assertRaisesRegex(ValueError, "malformed VC JSON"):
            verify_vc("{not-json")

    def test_non_did_key_vc_is_rejected_offline(self):
        non_did_key = {
            "issuer": "did:web:example.com",
            "proof": {"verificationMethod": "did:web:example.com#key-1"},
        }

        with self.assertRaisesRegex(ValueError, "requires a did:key issuer"):
            verify_vc(json.dumps(non_did_key))

    def test_matching_statement_rdfc_cid(self):
        self.assertTrue(verify_statement_rdfc_cid(json.dumps(self.statement)))

    def test_tampered_statement_rdfc_cid(self):
        tampered = {**self.statement, "timestamp": "2026-07-13T00:00:00Z"}
        self.assertFalse(verify_statement_rdfc_cid(json.dumps(tampered)))

    def test_malformed_statement_json(self):
        with self.assertRaisesRegex(ValueError, "malformed lineage statement JSON"):
            verify_statement_rdfc_cid("{not-json")


if __name__ == "__main__":
    unittest.main()
