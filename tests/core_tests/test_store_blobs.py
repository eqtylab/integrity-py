import os
import unittest
from pathlib import Path

from eqty_sdk import get_cid_for_bytes, purge_blob_store
from tests import get_config_dir, setup_sdk


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
        purge_blob_store()

        get_cid_for_bytes(test_bytes, False)
        self.assertFalse(os.path.exists(blob_path), "Arg Failure. Blob should not be stored")

        get_cid_for_bytes(test_bytes, True)
        self.assertTrue(os.path.exists(blob_path), "Arg Failure. Blob should be stored")


if __name__ == "__main__":
    unittest.main()
