import json
import tempfile
import unittest

from eqty_sdk._rust import (
    manifest,
    statements as eqty_core_statements,
)
from tests.rust import core_init, enable_logging


class TestManifestImport(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        """Set up test fixtures once for the entire test class."""
        cls.temp_dir = tempfile.mkdtemp(prefix=f"{cls.__name__}_")
        cls.logger = enable_logging(False)
        core_init(cls.temp_dir)

    @classmethod
    def tearDownClass(cls):
        """Clean up test fixtures after all tests are done."""
        import shutil

        shutil.rmtree(cls.temp_dir)

    def test_import_manifest_empty(self):
        """Test importing an empty manifest."""
        empty_manifest = json.dumps(
            {"statements": {}, "blobs": {}, "version": "3.0", "contexts": {}}
        )

        self.logger.info(f"Empty manifest to import: {empty_manifest}")
        result = manifest.import_manifest(empty_manifest)

        self.assertIsInstance(result, dict)
        self.assertEqual(len(result), 0)
        (statements, _) = eqty_core_statements.retrieve_statements()
        eqty_core_statements.delete_statements()
        self.logger.info("Imported statements: %s", statements)
        self.assertListEqual(statements, [])

    def test_import_manifest_with_blobs(self):
        """Test importing manifest with base64 encoded blobs."""
        # Create manifest with base64 encoded blob data
        import base64

        test_content = b"test blob content"
        encoded_content = base64.b64encode(test_content).decode()

        manifest_with_blobs = json.dumps(
            {
                "version": "3.0",
                "contexts": {},
                "statements": {},
                "blobs": {"test_blob_cid": encoded_content},
            }
        )

        imported_blobs = manifest.import_manifest(manifest_with_blobs)

        self.assertIsInstance(imported_blobs, dict)
        self.logger.info("BLOBS: %s", imported_blobs)
        self.assertIn("test_blob_cid", imported_blobs)
        self.assertEqual(imported_blobs["test_blob_cid"], test_content)

    def test_import_manifest_invalid_json(self):
        """Test importing invalid JSON manifest."""
        invalid_manifest = "invalid json"

        with self.assertRaises(Exception):
            manifest.import_manifest(invalid_manifest)

    def test_import_manifest_missing_fields(self):
        """Test importing manifest with missing required fields."""
        incomplete_manifest = json.dumps(
            {
                "statements": {}
                # Missing blobs field
            }
        )

        with self.assertRaises(Exception):
            manifest.import_manifest(incomplete_manifest)

    def test_import_manifest(self):
        statement = {
            "@context": "urn:cid:bafkr4iagb4u7jqlwqrftw4mn3l634wmgatmpvvzqgntgxaaerzljhggvdu",
            "@id": "urn:cid:bagb6qaq6eczjqmpzapg5zq25klhldpiih5adj3soi2nsijcuxctcszpexc5ua",
            "@type": "ComputationRegistration",
            "input": [
                "urn:cid:bafkr4ibcl6e7kiy2pggcohuygv64wjudxkllx7tq664tbl2ehycl3hvd4m",
                "urn:cid:bafkr4ibhvohsmw5q3yst26iwbvahkyaypna7muexs2o6vrldg4ayemt2ee",
                "urn:cid:bagaachradlrhaifv5s36ni22vh7kexo5zb2z3r7phfycgbsw3ubtxut4dczq",
            ],
            "output": ["urn:cid:bafkr4ibagnvfrssnt4ezfkj7asynb5uid3l4fupq4sg6t4yunvfpovgtom"],
            "operatedBy": "did:key:zDnaeuQqEdtwNwfA8r1BFjjdics95kAKLHxjcQt35aUYXswLv",
            "executedOn": "did:key:zDnaeTSoP8KrL35vy8CVyEw8uwmBpvDMYGYK3Q12AgP3kK7Ky",
            "registeredBy": "did:key:zDnaeuYuGvB3ox3MSqA5K1axfqQu5U1Jz1JXya5Dh9F5ZhnqF",
            "timestamp": "2025-08-19T00:06:56Z",
        }
        demo_manifest = json.dumps(
            {"statements": {"cid": statement}, "blobs": {}, "version": "3.0", "contexts": {}}
        )

        result = manifest.import_manifest(demo_manifest)

        self.assertIsInstance(result, dict)
        (statements, _) = eqty_core_statements.retrieve_statements()
        eqty_core_statements.delete_statements()
        self.logger.info("Imported statements: %s", statements)
        self.assertListEqual(statements, [statement])
        self.assertIn(statement, statements)

    def test_import_manifest_multiple_blobs(self):
        """Test importing manifest with multiple blob types."""
        import base64

        text_content = "text blob"
        binary_content = bytes([0, 1, 2, 3, 4, 5])

        manifest_data = json.dumps(
            {
                "version": "",
                "contexts": {},
                "statements": {},
                "blobs": {
                    "text_blob": base64.b64encode(text_content.encode()).decode(),
                    "binary_blob": base64.b64encode(binary_content).decode(),
                },
            }
        )

        result = manifest.import_manifest(manifest_data)

        self.assertIsInstance(result, dict)
        self.assertEqual(len(result), 2)
        self.assertIn("text_blob", result)
        self.assertIn("binary_blob", result)


if __name__ == "__main__":
    unittest.main()
