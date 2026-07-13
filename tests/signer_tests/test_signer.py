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
        signer = Signer.new(name=name)
        loaded_signer = Signer.new(name=name, _load_if_exists=True)

        self.assertEqual(loaded_signer.name, name)
        self.assertEqual(loaded_signer.did_key, signer.did_key)

    def test_signer_new_with_name_raises_when_signer_exists(self):
        name = f"signer-{uuid.uuid4()}"
        Signer.new(name=name)

        with self.assertRaises(ValueError):
            Signer.new(name=name)

    def test_signer_new_load_if_exists_requires_name(self):
        with self.assertRaises(ValueError):
            Signer.new(_load_if_exists=True)

    def test_load_or_create_creates_then_reuses(self):
        name = f"signer-{uuid.uuid4()}"
        created = Signer.load_or_create(name=name)
        reused = Signer.load_or_create(name=name)

        self.assertEqual(reused.name, name)
        self.assertEqual(reused.did_key, created.did_key)

    def test_load_or_create_honours_algorithm_on_create(self):
        name = f"signer-{uuid.uuid4()}"
        signer = Signer.load_or_create(name=name, algorithm=SIGNER_ALGORITHMS.SECP256K1)
        self.assertTrue(signer.did_key.startswith("did:key:"))

    def test_load_returns_persisted_signer(self):
        name = f"signer-{uuid.uuid4()}"
        created = Signer.new(name=name)
        loaded = Signer.load(name)

        self.assertEqual(loaded.did_key, created.did_key)

    def test_load_raises_lookup_error_when_missing(self):
        with self.assertRaises(LookupError):
            Signer.load(f"missing-{uuid.uuid4()}")

    def test_load_if_exists_still_works_but_warns(self):
        """The old flag stays functional for one release, but must warn."""
        name = f"signer-{uuid.uuid4()}"
        created = Signer.new(name=name)

        with self.assertWarns(DeprecationWarning) as ctx:
            reused = Signer.new(name=name, _load_if_exists=True)

        self.assertEqual(reused.did_key, created.did_key)
        self.assertIn("load_or_create", str(ctx.warning))

    def test_new_does_not_warn(self):
        import warnings

        with warnings.catch_warnings():
            warnings.simplefilter("error", DeprecationWarning)
            Signer.new(name=f"signer-{uuid.uuid4()}")

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

    def test_set_active_signer_by_instance(self):
        signer = Signer.new(SIGNER_ALGORITHMS.ED25519)
        set_active_signer(signer)

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


if __name__ == "__main__":
    unittest.main()
