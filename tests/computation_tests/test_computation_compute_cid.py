import unittest
from pathlib import Path

from eqty_sdk import CID, Computation
from tests import setup_sdk


class TestComputationCompute(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_default(self):
        """Test that the computation is None."""
        computation = Computation.new()
        self.assertIsNone(computation._computation_cid)

    def test_cid_str(self):
        """Test setting compute CID."""
        cid = CID("urn:cid:1")
        computation = Computation.new().set_computation_cid(cid)
        self.assertIsNotNone(computation._computation_cid)
        self.assertEqual(computation._computation_cid, cid)

    def test_cid(self):
        """Test setting compute CID."""
        cid = CID("urn:cid:4")
        computation = Computation.new().set_computation_cid(cid)
        self.assertIsNotNone(computation._computation_cid)
        self.assertEqual(computation._computation_cid, cid)

    def test_path_str(self):
        """Test path computations."""
        test_path = "tests/fixtures/assets/datasets/file/file_text.txt"

        file_path = "tests/fixtures/assets/datasets/file/file_text.txt.cid"
        with open(file_path, "r") as file:
            cid = CID(file.read().rstrip())
        cid

        computation = Computation.new().set_computation_path(test_path)
        self.assertIsNotNone(computation._computation_cid)
        self.assertEqual(computation._computation_cid, cid)

    def test_path(self):
        """Test path computations."""
        test_path = Path("tests/fixtures/assets/datasets/file/file_text.txt")

        file_path = "tests/fixtures/assets/datasets/file/file_text.txt.cid"
        with open(file_path, "r") as file:
            cid = CID(file.read().rstrip())
        cid

        computation = Computation.new().set_computation_path(test_path)
        self.assertIsNotNone(computation._computation_cid)
        self.assertEqual(computation._computation_cid, cid)

    def test_obj(self):
        test_obj = "hello world"
        cid = CID("bafkr4igxjga67jykbseaxdmmdgc5a5o3zp3htom2l6mrjznk7fvyggu6eq")
        computation = Computation.new().set_computation_object(test_obj)
        self.assertIsNotNone(computation._computation_cid)
        self.assertEqual(computation._computation_cid, cid)


if __name__ == "__main__":
    unittest.main()
