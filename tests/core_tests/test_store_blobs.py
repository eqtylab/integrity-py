import os
import unittest
from pathlib import Path

from eqty_sdk import config, get_cid_for_bytes
from tests import clean_blobs, get_config_dir, setup_sdk


class TestCoreStore(unittest.TestCase):
    def setUp(self):
        """Runs before every test function."""
        setup_sdk()

    def test_01_store_arg(self):
        """Check that setting the 'store' argument works."""
        test_bytes = b"test_store_arg"
        blob_path = Path(get_config_dir()).joinpath(
            "blobs/bafkr4ihmzr3yigdvuteldkbz2x65ngwtgqog4vcep5lcvp7whk2t7epzry"
        )
        clean_blobs()

        get_cid_for_bytes(test_bytes, False)
        self.assertFalse(os.path.exists(blob_path), "Arg Failure. Blob should not be stored")

        get_cid_for_bytes(test_bytes, True)
        self.assertTrue(os.path.exists(blob_path), "Arg Failure. Blob should be stored")

    def test_02_config_setting(self):
        """Check that setting the Config 'store_all_blobs' setting works."""
        test_bytes = b"goodbye world"
        blob_path = Path(get_config_dir()).joinpath(
            "blobs/bafkr4igqqhs24wo3mlyaus4nvn2hsm2hp2ouf7rjqwliq5oelkqw7gkuxa"
        )
        clean_blobs()

        config.set_store_all_blobs(False)
        get_cid_for_bytes(test_bytes)
        self.assertFalse(os.path.exists(blob_path), "Config Failure. Blob should not be stored")

        config.set_store_all_blobs(True)
        get_cid_for_bytes(test_bytes)

        self.assertTrue(os.path.exists(blob_path), "Config Failure. Blob should be stored")
        with open(blob_path, "rb") as blob:
            data = blob.read()
            self.assertEqual(data, test_bytes, "Blob stored with incorrect format")


if __name__ == "__main__":
    unittest.main()
