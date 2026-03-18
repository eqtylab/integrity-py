import unittest
from pathlib import Path
from uuid import uuid4

from eqty_sdk import SIGNER_ALGORITHMS, Model, Signer, set_active_signer
from tests import get_config_dir, get_statement_count_by_type, setup_sdk


class TestModelSigningSigstore(unittest.TestCase):
    def test_model_directory_writes_sigstore_bundle(self):
        setup_sdk()

        before = get_statement_count_by_type("CredentialSigstoreBundleRegistration") or 0

        default_signer = Signer.from_private_key(
            algorithm=SIGNER_ALGORITHMS.ED25519,
            private_key="eHb22WNFvUXihogn8fubQjW7hHEqwY3fEKt745V4xXg=",
        )
        model_signer = Signer.new(SIGNER_ALGORITHMS.SECP256R1)

        try:
            set_active_signer(model_signer)

            model_dir = get_config_dir() / f"model-signing-{uuid4().hex}"
            model_dir.mkdir(parents=True, exist_ok=True)
            (model_dir / "weights.txt").write_text("unit-test", encoding="utf-8")

            Model.from_path(Path(model_dir), _store=False, name="Model Dir", _skip_proof=True)
        finally:
            set_active_signer(default_signer)

        after = get_statement_count_by_type("CredentialSigstoreBundleRegistration") or 0
        self.assertEqual(
            after,
            before + 1,
            "Expected a sigstore bundle statement to be stored after model directory creation",
        )


if __name__ == "__main__":
    unittest.main()
