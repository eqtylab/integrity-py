import json
import tempfile
import unittest
from pathlib import Path

from eqty_sdk._rust import get_cid_for_path, statements
from tests.rust import core_init, enable_logging

DEFAULT_GRAPH_ID = "00000000-0000-0000-0000-000000000000"


class PublicStatementTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.logger = enable_logging(True)
        cls.temp_dir = tempfile.mkdtemp(prefix=f"{cls.__name__}_")
        core_init(cls.temp_dir)

    @classmethod
    def tearDownClass(cls):
        import shutil

        shutil.rmtree(cls.temp_dir)

    def test_add_association_statement(self):
        ids = statements.add_association_statement(
            "urn:cid:assoc-subject",
            "urn:cid:assoc-target",
            skip_proof=True,
        )
        self.assertEqual(1, len(ids))
        self.assertTrue(ids[0].startswith("urn:cid:"))

    def test_add_data_statement(self):
        ids = statements.add_data_statement(["urn:cid:data-1"], skip_proof=True)
        self.assertEqual(1, len(ids))
        self.assertTrue(ids[0].startswith("urn:cid:"))

    def test_add_computation_statement(self):
        ids = statements.add_computation_statement(
            ["urn:cid:input-1"],
            ["urn:cid:output-1"],
            computation="urn:cid:compute-1",
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
        self.assertIsInstance(statement_id, str)
        self.assertTrue(statement_id.startswith("urn:cid:"))

    def test_register_statement_to_graph(self):
        did_ids = statements.add_did_statement(
            "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
            skip_proof=True,
        )
        statements.register_statement_to_graph(did_ids[0], DEFAULT_GRAPH_ID)

    def test_create_model_signing_statement(self):
        model_dir = Path(self.temp_dir) / "model"
        model_dir.mkdir(parents=True, exist_ok=True)
        (model_dir / "weights.txt").write_text("unit-test", encoding="utf-8")

        collection_cid = get_cid_for_path(model_dir)
        blobs_dir = Path(self.temp_dir) / "blobs"

        statement_id = statements.create_model_signing_statement(
            collection_cid,
            blobs_dir,
            "unit-test-model",
            False,
            [],
        )
        self.assertIsInstance(statement_id, str)
        self.assertTrue(statement_id.startswith("urn:cid:"))


if __name__ == "__main__":
    unittest.main()
