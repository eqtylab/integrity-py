import shutil
import tempfile
import unittest

import eqty_sdk._rust as core
from tests.rust import enable_logging


class SignerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp_dir = tempfile.mkdtemp(prefix=f"{cls.__name__}_")
        core.context.init(cls.temp_dir)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.temp_dir)

    def test_file_cid(self):
        # with self.assertRaises(RuntimeError) as context:
        #     core.signer.get_active_signer_did_key()
        # self.assertEqual(str(context.exception), "RuntimeError: No active signer available")

        signer = core.signer.create_new_signer("secp256k1")
        core.signer.set_active_signer(signer.name)
        did_key = core.signer.get_active_signer_did_key()
        self.assertEqual(signer.did_key, did_key)


if __name__ == "__main__":
    unittest.main()
    enable_logging(False)
