import json
import tempfile
import unittest
from pathlib import Path

from eqty_sdk._rust import (
    manifest,
    statements as eqty_core_statements,
)
from tests.rust import core_init, create_simple_graph, enable_logging


class TestManifestGenerate(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        """Set up test fixtures once for the entire test class."""
        cls.logger = enable_logging(False)
        cls.temp_dir = tempfile.mkdtemp(prefix=f"{cls.__name__}_")
        cls.blobs_dir = Path(cls.temp_dir) / "blobs"
        cls.blobs_dir.mkdir(exist_ok=True)
        core_init(cls.temp_dir)
        create_simple_graph()

    @classmethod
    def tearDownClass(cls):
        """Clean up test fixtures after all tests are done."""
        import shutil

        shutil.rmtree(cls.temp_dir)

    def test_generate_empty_statements(self):
        """Test manifest generation with empty statements list."""
        result = manifest.generate([], self.blobs_dir)

        self.assertIsInstance(result, str)
        manifest_json = json.loads(result)
        self.assertIn("statements", manifest_json)
        self.assertIn("blobs", manifest_json)
        self.assertEqual(len(manifest_json["statements"]), 0)
        self.assertNotIn("attributes", manifest_json)

    def test_generate_with_valid_statement(self):
        """Test manifest generation with valid statement data."""
        (statements, _) = eqty_core_statements.retrieve_graph()
        result = manifest.generate(statements, self.blobs_dir)

        self.assertIsInstance(result, str)
        manifest_json = json.loads(result)
        self.assertIn("statements", manifest_json)
        self.assertNotEqual(manifest_json["statements"], {})

    def test_generate_nonexistent_blobs_dir(self):
        """Test manifest generation with non-existent blobs directory."""
        nonexistent_dir = Path(self.temp_dir) / "nonexistent"

        result = manifest.generate([], nonexistent_dir)

        self.assertIsInstance(result, str)
        manifest_json = json.loads(result)
        self.assertIn("blobs", manifest_json)
        self.assertEqual(len(manifest_json["blobs"]), 0)

    def test_generate_with_attributes(self):
        """Test manifest generation with attributes."""
        (statements, attributes) = eqty_core_statements.retrieve_statements()
        result = manifest.generate(statements, self.blobs_dir, attributes, False)

        self.assertIsInstance(result, str)
        manifest_json = json.loads(result)
        self.assertIn("attributes", manifest_json)
        self.logger.info("ATTRIBUTES: %s", manifest_json["attributes"])
        self.assertEqual(
            manifest_json["attributes"][
                "urn:cid:bagb6qaq6ec5lqkkz2qfc557j4fg3eetknd5scfldstsvm6pi32bh53hmkkuye"
            ],
            {},
        )


if __name__ == "__main__":
    unittest.main()
