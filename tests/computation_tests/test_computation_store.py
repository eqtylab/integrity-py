import os
import unittest
from pathlib import Path

from eqty_sdk import Computation, purge_blob_store
from tests import get_config_dir, setup_sdk

# CID for the string "hello world", as verified in test_computation_inputs.py.
HELLO_WORLD_BLOB_PATH = "blobs/bafkr4igxjga67jykbseaxdmmdgc5a5o3zp3htom2l6mrjznk7fvyggu6eq"


class TestComputationStore(unittest.TestCase):
    def setUp(self):
        """Runs before every test function."""
        setup_sdk()

    def test_add_input_object_store_false(self):
        """Test that Computation.new(_store=False) does not persist input object blobs."""
        purge_blob_store()
        blob_path = Path(get_config_dir()).joinpath(HELLO_WORLD_BLOB_PATH)

        Computation.new(_store=False).add_input_object("hello world")

        self.assertFalse(os.path.exists(blob_path), "Blob should not be stored when _store=False")

    def test_add_input_object_store_true(self):
        """Test that Computation.new(_store=True) persists input object blobs."""
        purge_blob_store()
        blob_path = Path(get_config_dir()).joinpath(HELLO_WORLD_BLOB_PATH)

        Computation.new(_store=True).add_input_object("hello world")

        self.assertTrue(os.path.exists(blob_path), "Blob should be stored when _store=True")

    def test_add_output_object_store_true(self):
        """Test that Computation.new(_store=True) persists output object blobs."""
        purge_blob_store()
        blob_path = Path(get_config_dir()).joinpath(HELLO_WORLD_BLOB_PATH)

        Computation.new(_store=True).add_output_object("hello world")

        self.assertTrue(os.path.exists(blob_path), "Blob should be stored when _store=True")

    def test_set_computation_object_store_true(self):
        """Test that Computation.new(_store=True) persists the computation object blob."""
        purge_blob_store()
        blob_path = Path(get_config_dir()).joinpath(HELLO_WORLD_BLOB_PATH)

        Computation.new(_store=True).set_computation_object("hello world")

        self.assertTrue(os.path.exists(blob_path), "Blob should be stored when _store=True")


if __name__ == "__main__":
    unittest.main()
