import tempfile
import unittest

from eqty_sdk._rust import statements
from tests.rust import core_init, enable_logging


class AssociationStatementTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        """Set up test fixtures once for the entire test class."""
        cls.logger = enable_logging(True)
        cls.temp_dir = tempfile.mkdtemp(prefix=f"{cls.__name__}_")
        core_init(cls.temp_dir)

    @classmethod
    def tearDownClass(cls):
        """Clean up test fixtures after all tests are done."""
        import shutil

        shutil.rmtree(cls.temp_dir)

    def test_create_association_statement(self):
        """Test creating an association statement."""
        timestamp = "2025-10-05T14:53:29Z"
        subject = "urn:cid:bafkr4iapekacqnwhacmcnugumd6o7b3pmbomvt37qwbfwqxe7ygbfc3atu"
        association = "urn:cid:bafkr4iapekacqnwhacmcnugumd6o7b3pmbomvt37qwbfwqxe7ygbfc3atv"

        # Create the association statement
        statement_id = statements.create_association_statement(subject, association, timestamp)

        # Verify the statement ID is returned
        self.assertIsNotNone(statement_id, "Statement ID should not be None")
        self.assertIsInstance(statement_id, str, "Statement ID should be a string")
        self.assertTrue(statement_id.startswith("urn:cid:"), "Statement ID should be a CID URN")

        # Add attributes to track this test
        test_attributes = {"test_name": "test_create_association_statement", "type": "association"}
        statements.add_attributes_to_statements([statement_id], test_attributes)

        # Retrieve and verify the statement exists
        (retrieved_statements, attributes) = statements.retrieve_statements(
            "attributes.test_name == 'test_create_association_statement'"
        )

        self.assertEqual(len(retrieved_statements), 1, "Should retrieve exactly one statement")
        self.assertIn(statement_id, attributes, "Statement should have attributes")
        self.assertEqual(
            attributes[statement_id]["test_name"],
            "test_create_association_statement",
            "Should have correct test_name attribute",
        )

    def test_association_statement_with_cid_strings(self):
        """Test creating association statement with plain CID strings (no URN prefix)."""
        timestamp = "2025-10-05T15:00:00Z"
        subject_cid = "bafkr4iapekacqnwhacmcnugumd6o7b3pmbomvt37qwbfwqxe7ygbfc3atu"
        association_cid = "bafkr4iapekacqnwhacmcnugumd6o7b3pmbomvt37qwbfwqxe7ygbfc3atv"

        # Create the association statement with plain CID strings
        statement_id = statements.create_association_statement(
            subject_cid, association_cid, timestamp
        )

        # Verify the statement was created successfully
        self.assertIsNotNone(statement_id, "Statement ID should not be None")
        self.assertIsInstance(statement_id, str, "Statement ID should be a string")
        self.assertTrue(statement_id.startswith("urn:cid:"), "Statement ID should be a CID URN")

    def test_association_statement_with_did(self):
        """Test creating association statement with DID identifiers."""
        timestamp = "2025-10-05T15:05:00Z"
        subject_did = "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp"
        association_did = "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG"

        # Create the association statement with DIDs
        statement_id = statements.create_association_statement(
            subject_did, association_did, timestamp
        )

        # Verify the statement was created successfully
        self.assertIsNotNone(statement_id, "Statement ID should not be None")
        self.assertIsInstance(statement_id, str, "Statement ID should be a string")
        self.assertTrue(statement_id.startswith("urn:cid:"), "Statement ID should be a CID URN")


if __name__ == "__main__":
    unittest.main()
