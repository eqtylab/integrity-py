import unittest
from pathlib import Path

from eqty_sdk import (
    CID,
    AssetType,
    Attribution,
    Benchmark,
    Certificate,
    Code,
    Custom,
    Database,
    Dataset,
    Document,
    Media,
    Model,
    Token,
)
from tests import setup_sdk


class AssetTypes(unittest.TestCase):
    """Checks that each asset type gets the correct asset_type set."""

    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_from_obj_type(self):
        """Test the from_object constructor."""
        test_obj = 6

        asset = Attribution.from_object(test_obj)
        self.assertEqual(asset.asset_type, "Attribution", "Attribution asset_type mismatch")

        asset = Benchmark.from_object(test_obj)
        self.assertEqual(asset.asset_type, "Benchmark", "Benchmark asset_type mismatch")

        asset = Certificate.from_object(test_obj)
        self.assertEqual(asset.asset_type, "Certificate", "Certificate asset_type mismatch")

        asset = Code.from_object(test_obj)
        self.assertEqual(asset.asset_type, "Code", "Code asset_type mismatch")

        asset = Custom.from_object(test_obj, AssetType.CUSTOM)
        self.assertEqual(asset.asset_type, "Custom", "Custom asset_type mismatch")

        asset = Database.from_object(test_obj)
        self.assertEqual(asset.asset_type, "Database", "Database asset_type mismatch")

        asset = Dataset.from_object(test_obj)
        self.assertEqual(asset.asset_type, "Dataset", "Dataset asset_type mismatch")

        asset = Document.from_object(test_obj)
        self.assertEqual(asset.asset_type, "Document", "Document asset_type mismatch")

        asset = Media.from_object(test_obj)
        self.assertEqual(asset.asset_type, "Media", "Media asset_type mismatch")

        asset = Model.from_object(test_obj)
        self.assertEqual(asset.asset_type, "Model", "Model asset_type mismatch")

        asset = Token.from_object(test_obj)
        self.assertEqual(asset.asset_type, "Token", "Token asset_type mismatch")

    def test_from_file_type(self):
        """Test the from_path constructor."""
        test_obj = Path("tests/fixtures/assets/datasets/file/file_text.txt")

        asset = Attribution.from_path(test_obj)
        self.assertEqual(asset.asset_type, "Attribution", "Attribution asset_type mismatch")

        asset = Benchmark.from_path(test_obj)
        self.assertEqual(asset.asset_type, "Benchmark", "Benchmark asset_type mismatch")

        asset = Certificate.from_path(test_obj)
        self.assertEqual(asset.asset_type, "Certificate", "Certificate asset_type mismatch")

        asset = Code.from_path(test_obj)
        self.assertEqual(asset.asset_type, "Code", "Code asset_type mismatch")

        asset = Custom.from_path(test_obj, AssetType.CUSTOM)
        self.assertEqual(asset.asset_type, "Custom", "Custom asset_type mismatch")

        asset = Database.from_path(test_obj)
        self.assertEqual(asset.asset_type, "Database", "Database asset_type mismatch")

        asset = Dataset.from_path(test_obj)
        self.assertEqual(asset.asset_type, "Dataset", "Dataset asset_type mismatch")

        asset = Document.from_path(test_obj)
        self.assertEqual(asset.asset_type, "Document", "Document asset_type mismatch")

        asset = Media.from_path(test_obj)
        self.assertEqual(asset.asset_type, "Media", "Media asset_type mismatch")

        asset = Model.from_path(test_obj)
        self.assertEqual(asset.asset_type, "Model", "Model asset_type mismatch")

        asset = Token.from_path(test_obj)
        self.assertEqual(asset.asset_type, "Token", "Token asset_type mismatch")

    def test_from_cid_type(self):
        """Test the from_cid constructor."""
        test_obj = CID("bafkr4ic6sphckk3a5x2fmgdeqod6tvv6k253q37icpfsk73wp2f524pok2")

        asset = Attribution.from_cid(test_obj)
        self.assertEqual(asset.asset_type, "Attribution", "Attribution asset_type mismatch")

        asset = Benchmark.from_cid(test_obj)
        self.assertEqual(asset.asset_type, "Benchmark", "Benchmark asset_type mismatch")

        asset = Certificate.from_cid(test_obj)
        self.assertEqual(asset.asset_type, "Certificate", "Certificate asset_type mismatch")

        asset = Code.from_cid(test_obj)
        self.assertEqual(asset.asset_type, "Code", "Code asset_type mismatch")

        asset = Custom.from_cid(test_obj, AssetType.CUSTOM)
        self.assertEqual(asset.asset_type, "Custom", "Custom asset_type mismatch")

        asset = Database.from_cid(test_obj)
        self.assertEqual(asset.asset_type, "Database", "Database asset_type mismatch")

        asset = Dataset.from_cid(test_obj)
        self.assertEqual(asset.asset_type, "Dataset", "Dataset asset_type mismatch")

        asset = Document.from_cid(test_obj)
        self.assertEqual(asset.asset_type, "Document", "Document asset_type mismatch")

        asset = Media.from_cid(test_obj)
        self.assertEqual(asset.asset_type, "Media", "Media asset_type mismatch")

        asset = Model.from_cid(test_obj)
        self.assertEqual(asset.asset_type, "Model", "Model asset_type mismatch")

        asset = Token.from_cid(test_obj)
        self.assertEqual(asset.asset_type, "Token", "Token asset_type mismatch")


if __name__ == "__main__":
    unittest.main()
