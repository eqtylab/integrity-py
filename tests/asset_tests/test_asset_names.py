import unittest
from pathlib import Path

from eqty_sdk.asset import (
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
from eqty_sdk.types import CID
from tests import setup_sdk


class AssetNames(unittest.TestCase):
    """Checks that each asset type gets the correct name."""

    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_default_name_from_object(self):
        test_obj = 6

        asset = Attribution.from_object(test_obj)
        self.assertEqual(asset.name, "Attribution-kxgi", "Attribution name mismatch")

        asset = Benchmark.from_object(test_obj)
        self.assertEqual(asset.name, "Benchmark-kxgi", "Benchmark name mismatch")

        asset = Certificate.from_object(test_obj)
        self.assertEqual(asset.name, "Certificate-kxgi", "Certificate name mismatch")

        asset = Code.from_object(test_obj)
        self.assertEqual(asset.name, "Code-kxgi", "Code name mismatch")

        asset = Custom.from_object(test_obj, AssetType.CUSTOM)
        self.assertEqual(asset.name, "Custom-kxgi", "Custom name mismatch")

        asset = Database.from_object(test_obj)
        self.assertEqual(asset.name, "Database-kxgi", "Database name mismatch")

        asset = Dataset.from_object(test_obj)
        self.assertEqual(asset.name, "Dataset-kxgi", "Dataset name mismatch")

        asset = Document.from_object(test_obj)
        self.assertEqual(asset.name, "Document-kxgi", "Document name mismatch")

        asset = Media.from_object(test_obj)
        self.assertEqual(asset.name, "Media-kxgi", "Media name mismatch")

        asset = Model.from_object(test_obj)
        self.assertEqual(asset.name, "Model-kxgi", "Model name mismatch")

        asset = Token.from_object(test_obj)
        self.assertEqual(asset.name, "Token-kxgi", "Token name mismatch")

    def test_default_name_from_file_type(self):
        test_obj = Path("tests/fixtures/assets/datasets/file/file_text.txt")

        asset = Attribution.from_path(test_obj)
        self.assertEqual(asset.name, "Attribution-mmi4", "Attribution name mismatch")

        asset = Benchmark.from_path(test_obj)
        self.assertEqual(asset.name, "Benchmark-mmi4", "Benchmark name mismatch")

        asset = Certificate.from_path(test_obj)
        self.assertEqual(asset.name, "Certificate-mmi4", "Certificate name mismatch")

        asset = Code.from_path(test_obj)
        self.assertEqual(asset.name, "Code-mmi4", "Code name mismatch")

        asset = Custom.from_path(test_obj, AssetType.CUSTOM)
        self.assertEqual(asset.name, "Custom-mmi4", "Custom name mismatch")

        asset = Database.from_path(test_obj)
        self.assertEqual(asset.name, "Database-mmi4", "Database name mismatch")

        asset = Dataset.from_path(test_obj)
        self.assertEqual(asset.name, "Dataset-mmi4", "Dataset name mismatch")

        asset = Document.from_path(test_obj)
        self.assertEqual(asset.name, "Document-mmi4", "Document name mismatch")

        asset = Media.from_path(test_obj)
        self.assertEqual(asset.name, "Media-mmi4", "Media name mismatch")

        asset = Model.from_path(test_obj)
        self.assertEqual(asset.name, "Model-mmi4", "Model name mismatch")

        asset = Token.from_path(test_obj)
        self.assertEqual(asset.name, "Token-mmi4", "Token name mismatch")

    def test_default_name_from_cid_type(self):
        test_obj = CID("bafkr4ic6sphckk3a5x2fmgdeqod6tvv6k253q37icpfsk73wp2f524pok2")

        asset = Attribution.from_cid(test_obj)
        self.assertEqual(asset.name, "Attribution-pok2", "Attribution name mismatch")

        asset = Benchmark.from_cid(test_obj)
        self.assertEqual(asset.name, "Benchmark-pok2", "Benchmark name mismatch")

        asset = Certificate.from_cid(test_obj)
        self.assertEqual(asset.name, "Certificate-pok2", "Certificate name mismatch")

        asset = Code.from_cid(test_obj)
        self.assertEqual(asset.name, "Code-pok2", "Code name mismatch")

        asset = Custom.from_cid(test_obj, AssetType.CUSTOM)
        self.assertEqual(asset.name, "Custom-pok2", "Custom name mismatch")

        asset = Database.from_cid(test_obj)
        self.assertEqual(asset.name, "Database-pok2", "Database name mismatch")

        asset = Dataset.from_cid(test_obj)
        self.assertEqual(asset.name, "Dataset-pok2", "Dataset name mismatch")

        asset = Document.from_cid(test_obj)
        self.assertEqual(asset.name, "Document-pok2", "Document name mismatch")

        asset = Media.from_cid(test_obj)
        self.assertEqual(asset.name, "Media-pok2", "Media name mismatch")

        asset = Model.from_cid(test_obj)
        self.assertEqual(asset.name, "Model-pok2", "Model name mismatch")

        asset = Token.from_cid(test_obj)
        self.assertEqual(asset.name, "Token-pok2", "Token name mismatch")

    def test_custom_name_from_obj(self):
        test_obj = Path("tests/fixtures/assets/datasets/file/file_text.txt")

        asset = Attribution.from_object(test_obj, name="obj")
        self.assertEqual(asset.name, "obj", "Attribution name mismatch")

        asset = Benchmark.from_object(test_obj, name="obj")
        self.assertEqual(asset.name, "obj", "Benchmark name mismatch")

        asset = Certificate.from_object(test_obj, name="obj")
        self.assertEqual(asset.name, "obj", "Certificate name mismatch")

        asset = Code.from_object(test_obj, name="obj")
        self.assertEqual(asset.name, "obj", "Code name mismatch")

        asset = Custom.from_object(test_obj, AssetType.CUSTOM, name="obj")
        self.assertEqual(asset.name, "obj", "Custom name mismatch")

        asset = Database.from_object(test_obj, name="obj")
        self.assertEqual(asset.name, "obj", "Database name mismatch")

        asset = Dataset.from_object(test_obj, name="obj")
        self.assertEqual(asset.name, "obj", "Dataset name mismatch")

        asset = Document.from_object(test_obj, name="obj")
        self.assertEqual(asset.name, "obj", "Document name mismatch")

        asset = Media.from_object(test_obj, name="obj")
        self.assertEqual(asset.name, "obj", "Media name mismatch")

        asset = Model.from_object(test_obj, name="obj")
        self.assertEqual(asset.name, "obj", "Model name mismatch")

        asset = Token.from_object(test_obj, name="obj")
        self.assertEqual(asset.name, "obj", "Token name mismatch")

    def test_custom_name_from_path(self):
        test_obj = Path("tests/fixtures/assets/datasets/file/file_text.txt")

        asset = Attribution.from_path(test_obj, name="path")
        self.assertEqual(asset.name, "path", "Attribution name mismatch")

        asset = Benchmark.from_path(test_obj, name="path")
        self.assertEqual(asset.name, "path", "Benchmark name mismatch")

        asset = Certificate.from_path(test_obj, name="path")
        self.assertEqual(asset.name, "path", "Certificate name mismatch")

        asset = Code.from_path(test_obj, name="path")
        self.assertEqual(asset.name, "path", "Code name mismatch")

        asset = Custom.from_path(test_obj, AssetType.CUSTOM, name="path")
        self.assertEqual(asset.name, "path", "Custom name mismatch")

        asset = Database.from_path(test_obj, name="path")
        self.assertEqual(asset.name, "path", "Database name mismatch")

        asset = Dataset.from_path(test_obj, name="path")
        self.assertEqual(asset.name, "path", "Dataset name mismatch")

        asset = Document.from_path(test_obj, name="path")
        self.assertEqual(asset.name, "path", "Document name mismatch")

        asset = Media.from_path(test_obj, name="path")
        self.assertEqual(asset.name, "path", "Media name mismatch")

        asset = Model.from_path(test_obj, name="path")
        self.assertEqual(asset.name, "path", "Model name mismatch")

        asset = Token.from_path(test_obj, name="path")
        self.assertEqual(asset.name, "path", "Token name mismatch")

    def test_custom_name_from_cid(self):
        test_obj = CID("bafkr4ic6sphckk3a5x2fmgdeqod6tvv6k253q37icpfsk73wp2f524pok2")

        asset = Attribution.from_cid(test_obj, name="cid")
        self.assertEqual(asset.name, "cid", "Attribution name mismatch")

        asset = Benchmark.from_cid(test_obj, name="cid")
        self.assertEqual(asset.name, "cid", "Benchmark name mismatch")

        asset = Certificate.from_cid(test_obj, name="cid")
        self.assertEqual(asset.name, "cid", "Certificate name mismatch")

        asset = Code.from_cid(test_obj, name="cid")
        self.assertEqual(asset.name, "cid", "Code name mismatch")

        asset = Custom.from_cid(test_obj, AssetType.CUSTOM, name="cid")
        self.assertEqual(asset.name, "cid", "Custom name mismatch")

        asset = Database.from_cid(test_obj, name="cid")
        self.assertEqual(asset.name, "cid", "Database name mismatch")

        asset = Dataset.from_cid(test_obj, name="cid")
        self.assertEqual(asset.name, "cid", "Dataset name mismatch")

        asset = Document.from_cid(test_obj, name="cid")
        self.assertEqual(asset.name, "cid", "Document name mismatch")

        asset = Media.from_cid(test_obj, name="cid")
        self.assertEqual(asset.name, "cid", "Media name mismatch")

        asset = Model.from_cid(test_obj, name="cid")
        self.assertEqual(asset.name, "cid", "Model name mismatch")

        asset = Token.from_cid(test_obj, name="cid")
        self.assertEqual(asset.name, "cid", "Token name mismatch")


if __name__ == "__main__":
    unittest.main()
