import os
import unittest
import uuid

from eqty_sdk import SIGNER_ALGORITHMS, Signer, set_active_signer
from tests import setup_sdk


class SignerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_signer_new_default(self):
        signer = Signer.new()
        self.assertIsNotNone(signer)
        self.assertTrue(signer.did_key.startswith("did:key:"))
        self.assertTrue(signer.name)

    def test_signer_new_with_algorithm(self):
        signer = Signer.new(SIGNER_ALGORITHMS.ED25519)
        self.assertIsNotNone(signer)
        self.assertTrue(signer.did_key.startswith("did:key:"))
        self.assertTrue(signer.name)

    def test_signer_new_with_name_loads_existing_when_requested(self):
        name = f"signer-{uuid.uuid4()}"
        signer = Signer.new(name)
        loaded_signer = Signer.new(name, _load_if_exists=True)

        self.assertEqual(loaded_signer.name, name)
        self.assertEqual(loaded_signer.did_key, signer.did_key)

    def test_signer_new_with_name_raises_when_signer_exists(self):
        name = f"signer-{uuid.uuid4()}"
        Signer.new(name)

        with self.assertRaises(ValueError):
            Signer.new(name)

    def test_signer_new_load_if_exists_requires_name(self):
        with self.assertRaises(ValueError):
            Signer.new(_load_if_exists=True)

    def test_signer_from_private_key(self):
        signer = Signer.from_private_key(
            algorithm=SIGNER_ALGORITHMS.ED25519,
            private_key="eHb22WNFvUXihogn8fubQjW7hHEqwY3fEKt745V4xXg=",
        )
        self.assertTrue(signer.name)
        self.assertTrue(signer.did_key.startswith("did:key:"))

    def test_signer_from_private_key_loads_existing_named_signer(self):
        name = f"signer-{uuid.uuid4()}"
        signer = Signer.from_private_key(
            algorithm=SIGNER_ALGORITHMS.ED25519,
            private_key="eHb22WNFvUXihogn8fubQjW7hHEqwY3fEKt745V4xXg=",
            name=name,
        )
        loaded_signer = Signer.from_private_key(
            algorithm=SIGNER_ALGORITHMS.SECP256K1,
            private_key="ignored-when-loading",
            name=name,
            _load_if_exists=True,
        )

        self.assertEqual(loaded_signer.name, name)
        self.assertEqual(loaded_signer.did_key, signer.did_key)

    def test_signer_from_private_key_load_if_exists_requires_name(self):
        with self.assertRaises(ValueError):
            Signer.from_private_key(
                algorithm=SIGNER_ALGORITHMS.ED25519,
                private_key="eHb22WNFvUXihogn8fubQjW7hHEqwY3fEKt745V4xXg=",
                _load_if_exists=True,
            )

    def test_set_active_signer_by_instance_and_name(self):
        signer = Signer.new(SIGNER_ALGORITHMS.ED25519)
        set_active_signer(signer)
        set_active_signer(signer.name)

    def test_auth_service_requires_api_key_env(self):
        if "EQTY_API_KEY" in os.environ:
            del os.environ["EQTY_API_KEY"]

        with self.assertRaises(Exception):
            Signer.auth_service("http://localhost:9999")

    @unittest.skipUnless(
        os.getenv("EQTY_TEST_VCOMP", "").lower() in ("true", "1", "yes", "on"),
        "EQTY_TEST_VCOMP not enabled",
    )
    def test_vcomp_notary(self):
        signer = Signer.vcomp_notary("http://localhost:8066")
        self.assertTrue(signer.did_key.startswith("did:key:"))

    @unittest.skipUnless(
        os.getenv("EQTY_TEST_AUTH_SERVICE", "").lower() in ("true", "1", "yes", "on"),
        "EQTY_TEST_AUTH_SERVICE not enabled",
    )
    def test_auth_service(self):
        signer = Signer.auth_service("http://localhost:9999")
        self.assertTrue(signer.did_key.startswith("did:key:"))

    @unittest.skipUnless(
        os.getenv("EQTY_TEST_YUBIHSM2", "").lower() in ("true", "1", "yes", "on"),
        "EQTY_TEST_YUBIHSM2 not enabled",
    )
    def test_yubihsm2(self):
        signer = Signer.yubihsm2(1, 1, "password")
        self.assertTrue(signer.did_key.startswith("did:key:"))


if __name__ == "__main__":
    unittest.main()
