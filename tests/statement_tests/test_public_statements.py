import json
import unittest

from eqty_sdk import (
    ASSOCIATION_TYPES,
    CID,
    DID,
    SIGNER_ALGORITHMS,
    UUID,
    Association,
    Signer,
    set_active_signer,
)
from eqty_sdk._rust import PyAssociationType, get_cid_for_path, statements
from tests import get_config_dir, setup_sdk


class PublicStatementTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_add_association_statement(self):
        ids = statements.add_association_statement(
            "urn:cid:assoc-subject",
            ["urn:cid:assoc-target"],
            PyAssociationType.Certifies,
            skip_proof=True,
        )
        self.assertEqual(1, len(ids))
        self.assertTrue(ids[0].startswith("urn:cid:"))

    def test_association_builder(self):
        ctx = setup_sdk().get_default_context()
        association = (
            Association.with_context(ctx)
            .new(CID("urn:cid:assoc-subject"), ASSOCIATION_TYPES.INCLUDES)
            .add_predicate(CID("urn:cid:assoc-target"))
            .finalize()
        )
        self.assertIsInstance(association, Association)

    def test_association_builder_accepts_did_and_uuid(self):
        ctx = setup_sdk().get_default_context()
        did = DID.from_did_string("did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp")
        association = (
            Association.with_context(ctx)
            .new(UUID("urn:uuid:123e4567-e89b-12d3-a456-426614174000"), ASSOCIATION_TYPES.CERTIFIES)
            .add_predicate(did)
            .add_predicate([UUID("123e4567-e89b-12d3-a456-426614174111")])
            .finalize()
        )
        self.assertIsInstance(association, Association)

    def test_association_builder_rejects_invalid_predicate(self):
        ctx = setup_sdk().get_default_context()
        association = Association.with_context(ctx).new(
            CID("urn:cid:assoc-subject"), ASSOCIATION_TYPES.CERTIFIES
        )
        with self.assertRaises(ValueError):
            association.add_predicate("urn:cid:assoc-target")

    def test_add_data_statement(self):
        ids = statements.add_data_statement([CID("urn:cid:data-1")], skip_proof=True)
        self.assertEqual(1, len(ids))
        self.assertTrue(ids[0].startswith("urn:cid:"))

    def test_add_computation_statement(self):
        ids = statements.add_computation_statement(
            [CID("urn:cid:input-1")],
            [CID("urn:cid:output-1")],
            computation=CID("urn:cid:compute-1"),
            operated_by="did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
            executed_on="device:unit-test",
            skip_proof=True,
        )
        self.assertEqual(1, len(ids))
        self.assertTrue(ids[0].startswith("urn:cid:"))

    def test_add_did_statement(self):
        ids = statements.add_did_statement(
            "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
            skip_proof=True,
        )
        self.assertEqual(1, len(ids))
        self.assertTrue(ids[0].startswith("urn:cid:"))

    def test_add_entity_statement(self):
        ids = statements.add_entity_statement(
            "urn:uuid:123e4567-e89b-12d3-a456-426614174000",
            skip_proof=True,
        )
        self.assertEqual(1, len(ids))
        self.assertTrue(ids[0].startswith("urn:cid:"))

    def test_add_governance_statement(self):
        ids = statements.add_governance_statement(
            "urn:cid:governance-subject",
            "urn:cid:governance-doc",
            skip_proof=True,
        )
        self.assertEqual(1, len(ids))
        self.assertTrue(ids[0].startswith("urn:cid:"))

    def test_add_metadata_statement(self):
        metadata = json.dumps({"name": "unit-test", "kind": "metadata"})
        ids = statements.add_metadata_statement(
            "urn:cid:meta-subject",
            metadata,
            skip_proof=True,
        )
        self.assertEqual(1, len(ids))
        self.assertTrue(ids[0].startswith("urn:cid:"))

    def test_add_storage_statement(self):
        ids = statements.add_storage_statement(
            "urn:cid:stored-data",
            "s3://bucket/path",
            operated_by="did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
            skip_proof=True,
        )
        self.assertEqual(1, len(ids))
        self.assertTrue(ids[0].startswith("urn:cid:"))

    def test_add_vc_statement(self):
        statement_id = statements.add_vc_statement("urn:cid:vc-subject")
        self.assertIsInstance(statement_id, CID)
        self.assertTrue(statement_id.startswith("urn:cid:"))

    @unittest.skip("Model signing interfered with directory ciding")
    def test_create_model_signing_statement(self):
        signer = Signer.new(SIGNER_ALGORITHMS.SECP256R1)
        set_active_signer(signer)

        model_dir = get_config_dir() / "model"
        model_dir.mkdir(parents=True, exist_ok=True)
        (model_dir / "weights.txt").write_text("unit-test", encoding="utf-8")

        collection_cid = get_cid_for_path(model_dir)
        blobs_dir = get_config_dir() / "blobs"

        statement_id = statements.create_model_signing_statement(
            str(collection_cid),
            blobs_dir,
            "unit-test-model",
            False,
            [],
        )
        self.assertIsInstance(statement_id, str)
        self.assertTrue(statement_id.startswith("urn:cid:"))


if __name__ == "__main__":
    unittest.main()
