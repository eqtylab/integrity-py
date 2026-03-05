import unittest
from pathlib import Path
from uuid import uuid4

from eqty_sdk import Model
from tests import get_config_dir, get_statement_count_by_type, setup_sdk


class TestModelSigningSigstore(unittest.TestCase):
    def test_model_directory_writes_sigstore_bundle(self):
        setup_sdk()

        before = get_statement_count_by_type("CredentialSigstoreBundleRegistration") or 0

        model_dir = get_config_dir() / f"model-signing-{uuid4().hex}"
        model_dir.mkdir(parents=True, exist_ok=True)
        (model_dir / "weights.txt").write_text("unit-test", encoding="utf-8")

        Model.from_path(Path(model_dir), store=False, name="Model Dir")

        after = get_statement_count_by_type("CredentialSigstoreBundleRegistration") or 0
        self.assertEqual(
            after,
            before + 1,
            "Expected a sigstore bundle statement to be stored after model directory creation",
        )


if __name__ == "__main__":
    unittest.main()
