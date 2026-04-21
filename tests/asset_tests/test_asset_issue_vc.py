import os
import unittest

from eqty_sdk import (
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


class AssetIssueVC(unittest.TestCase):
    """Tests the different ways to override issuing a vc."""

    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_from_object_default(self):
        """Check that by default a VC is issued."""
        data = 1

        asset = Benchmark.from_object(data)
        self.assertFalse(asset._skip_proof, "Benchmark default skip_proof failed")

        asset = Certificate.from_object(data)
        self.assertFalse(asset._skip_proof, "Certificate default skip_proof failed")

        asset = Code.from_object(data)
        self.assertFalse(asset._skip_proof, "Code default skip_proof failed")

        asset = Custom.from_object(data, "custom")
        self.assertFalse(asset._skip_proof, "Custom default skip_proof failed")

        asset = Dataset.from_object(data)
        self.assertFalse(asset._skip_proof, "Dataset default skip_proof failed")

        asset = Database.from_object(data)
        self.assertFalse(asset._skip_proof, "Database default skip_proof failed")

        asset = Document.from_object(data)
        self.assertFalse(asset._skip_proof, "Document default skip_proof failed")

        asset = Media.from_object(data)
        self.assertFalse(asset._skip_proof, "Media default skip_proof failed")

        asset = Model.from_object(data)
        self.assertFalse(asset._skip_proof, "Model default skip_proof failed")

        asset = Token.from_object(data)
        self.assertFalse(asset._skip_proof, "Token default skip_proof failed")

    def test_asset_override(self):
        """Check that explicitly not issuing a VC works."""
        data = 2

        asset = Benchmark.from_object(data, _skip_proof=True)
        self.assertTrue(asset._skip_proof, "Benchmark override skip_proof failed")

        asset = Certificate.from_object(data, _skip_proof=True)
        self.assertTrue(asset._skip_proof, "Certificate override skip_proof failed")

        asset = Code.from_object(data, _skip_proof=True)
        self.assertTrue(asset._skip_proof, "Code override skip_proof failed")

        asset = Custom.from_object(data, "custom", _skip_proof=True)
        self.assertTrue(asset._skip_proof, "Custom override skip_proof failed")

        asset = Database.from_object(data, _skip_proof=True)
        self.assertTrue(asset._skip_proof, "Database override skip_proof failed")

        asset = Dataset.from_object(data, _skip_proof=True)
        self.assertTrue(asset._skip_proof, "Dataset override skip_proof failed")

        asset = Document.from_object(data, _skip_proof=True)
        self.assertTrue(asset._skip_proof, "Document override skip_proof failed")

        asset = Media.from_object(data, _skip_proof=True)
        self.assertTrue(asset._skip_proof, "Media override skip_proof failed")

        asset = Model.from_object(data, _skip_proof=True)
        self.assertTrue(asset._skip_proof, "Model override skip_proof failed")

        asset = Token.from_object(data, _skip_proof=True)
        self.assertTrue(asset._skip_proof, "Token override skip_proof failed")

    def test_asset_env_var(self):
        """Check that the env var disables issuing a vc."""
        os.environ["EQTY_SKIP_PROOF"] = "FaLsE"
        data = 3

        asset = Benchmark.from_object(data)
        self.assertFalse(asset._skip_proof, "Benchmark env var disable skip_proof failed")

        asset = Certificate.from_object(data)
        self.assertFalse(asset._skip_proof, "Certificate env var disable skip_proof failed")

        asset = Code.from_object(data)
        self.assertFalse(asset._skip_proof, "Code env var disable skip_proof failed")

        asset = Custom.from_object(data, "custom")
        self.assertFalse(asset._skip_proof, "Custom env var disable skip_proof failed")

        asset = Database.from_object(data)
        self.assertFalse(asset._skip_proof, "Database env var disable skip_proof failed")

        asset = Dataset.from_object(data)
        self.assertFalse(asset._skip_proof, "Dataset env var disable skip_proof failed")

        asset = Document.from_object(data)
        self.assertFalse(asset._skip_proof, "Document env var disable skip_proof failed")

        asset = Media.from_object(data)
        self.assertFalse(asset._skip_proof, "Media env var disable skip_proof failed")

        asset = Model.from_object(data)
        self.assertFalse(asset._skip_proof, "Model env var disable skip_proof failed")

        asset = Token.from_object(data)
        self.assertFalse(asset._skip_proof, "Token env var disable skip_proof failed")
        os.environ["EQTY_SKIP_PROOF"] = "true"

    def test_asset_env_var_and_forced_vc(self):
        """Test that explicitly issuing a VC overrides the env var for disabling issuing a VC."""
        os.environ["EQTY_SKIP_PROOF"] = ""
        data = 4

        asset = Benchmark.from_object(data)
        self.assertFalse(asset._skip_proof, "Benchmark env var disable skip_proof failed")
        asset_vc = Benchmark.from_object(data, _skip_proof=True)
        self.assertTrue(asset_vc._skip_proof, "Benchmark env var explicit skip_proof failed")

        asset = Certificate.from_object(data)
        self.assertFalse(asset._skip_proof, "Certificate env var disable skip_proof failed")
        asset_vc = Certificate.from_object(data, _skip_proof=True)
        self.assertTrue(asset_vc._skip_proof, "Certificate env var explicit skip_proof failed")

        asset = Code.from_object(data)
        self.assertFalse(asset._skip_proof, "Code env var disable skip_proof failed")
        asset_vc = Code.from_object(data, _skip_proof=True)
        self.assertTrue(asset_vc._skip_proof, "Code env var explicit skip_proof failed")

        asset = Custom.from_object(data, "custom")
        self.assertFalse(asset._skip_proof, "Custom env var disable skip_proof failed")
        asset_vc = Custom.from_object(data, "custom", _skip_proof=True)
        self.assertTrue(asset_vc._skip_proof, "Custom env var explicit skip_proof failed")

        asset = Database.from_object(data)
        self.assertFalse(asset._skip_proof, "Database env var disable skip_proof failed")
        asset_vc = Database.from_object(data, _skip_proof=True)
        self.assertTrue(asset_vc._skip_proof, "Database env var explicit skip_proof failed")

        asset = Dataset.from_object(data)
        self.assertFalse(asset._skip_proof, "Dataset env var disable skip_proof failed")
        asset_vc = Dataset.from_object(data, _skip_proof=True)
        self.assertTrue(asset_vc._skip_proof, "Dataset env var explicit skip_proof failed")

        asset = Document.from_object(data)
        self.assertFalse(asset._skip_proof, "Document env var disable skip_proof failed")
        asset_vc = Document.from_object(data, _skip_proof=True)
        self.assertTrue(asset_vc._skip_proof, "Document env var explicit skip_proof failed")

        asset = Media.from_object(data)
        self.assertFalse(asset._skip_proof, "Media env var disable skip_proof failed")
        asset_vc = Media.from_object(data, _skip_proof=True)
        self.assertTrue(asset_vc._skip_proof, "Media env var explicit skip_proof failed")

        asset = Model.from_object(data)
        self.assertFalse(asset._skip_proof, "Model env var disable skip_proof failed")
        asset_vc = Model.from_object(data, _skip_proof=True)
        self.assertTrue(asset_vc._skip_proof, "Model env var explicit skip_proof failed")

        asset = Token.from_object(data)
        self.assertFalse(asset._skip_proof, "Token env var disable skip_proof failed")
        asset_vc = Token.from_object(data, _skip_proof=True)
        self.assertTrue(asset_vc._skip_proof, "Token env var explicit skip_proof failed")


if __name__ == "__main__":
    unittest.main()
