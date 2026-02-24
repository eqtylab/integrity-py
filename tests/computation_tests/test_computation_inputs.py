import unittest
from pathlib import Path

from eqty_sdk import CID, Computation
from tests import setup_sdk


class TestComputationInputs(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_default(self):
        """Test that the inputs is array is empty."""
        computation = Computation.new()
        self.assertEqual(len(computation._input_cids), 0)

    def test_cid_str(self):
        """Test setting CID input."""
        expected_cids = []
        cid = "urn:cid:1"
        computation = Computation.new().add_input_cid(cid)
        expected_cids.append(cid)
        self.assertEqual(len(computation._input_cids), 1, "Adding cid str error")
        self.assertEqual(computation._input_cids, expected_cids)

    def test_cid_str_list(self):
        """Test setting list of CID str input."""
        expected_cids = []
        cid = ["urn:cid:2", "urn:cid:3"]
        computation = Computation.new().add_input_cid(cid)
        expected_cids.extend(cid)
        self.assertEqual(len(computation._input_cids), 2, "Adding cid List[str] error")
        self.assertEqual(computation._input_cids, expected_cids)

    def test_cid(self):
        """Test setting CID input."""
        cid = CID("urn:cid:4")
        expected_cids = [cid.cid]
        computation = Computation.new().add_input_cid(cid)
        self.assertEqual(len(computation._input_cids), 1, "Adding cid CID error")
        self.assertEqual(computation._input_cids, expected_cids)

    def test_cid_list(self):
        """Test setting List[CID] input."""
        cid5 = CID("urn:cid:5")
        cid6 = CID("urn:cid:6")
        computation = Computation.new().add_input_cid([cid5, cid6])
        expected_cids = [cid5.cid, cid6.cid]
        self.assertEqual(len(computation._input_cids), 2, "Adding cid List[CID] error")
        self.assertEqual(computation._input_cids, expected_cids)

    def test_path(self):
        """Test path inputs."""
        test_path = "tests/fixtures/assets/datasets/file/file_text.txt"

        file_path = "tests/fixtures/assets/datasets/file/file_text.txt.cid"
        with open(file_path, "r") as file:
            cid = file.read().rstrip()
        cid

        computation = Computation.new().add_input_path(test_path)
        self.assertEqual(len(computation._input_cids), 1, "Adding path str error")
        self.assertEqual(computation._input_cids, [cid])

        computation = computation.add_input_path(Path(test_path))
        self.assertEqual(len(computation._input_cids), 2, "Adding path Path error")
        self.assertEqual(computation._input_cids, [cid, cid])

        computation = computation.add_input_path([test_path, test_path])
        self.assertEqual(len(computation._input_cids), 4, "Adding path List[str] error")
        self.assertEqual(computation._input_cids, [cid, cid, cid, cid])

        computation = computation.add_input_path([Path(test_path), Path(test_path)])
        self.assertEqual(len(computation._input_cids), 6, "Adding path List[Path] error")
        self.assertEqual(computation._input_cids, [cid, cid, cid, cid, cid, cid])

    def test_obj(self):
        test_obj = "hello world"
        correct_cid = "bafkr4igxjga67jykbseaxdmmdgc5a5o3zp3htom2l6mrjznk7fvyggu6eq"
        computation = Computation.new().add_input_object(test_obj)
        self.assertEqual(len(computation._input_cids), 1, "Adding obj error")
        self.assertEqual(computation._input_cids, [correct_cid])

    def test_obj_list(self):
        test_obj = "hello world"
        correct_cid = "bafkr4igxjga67jykbseaxdmmdgc5a5o3zp3htom2l6mrjznk7fvyggu6eq"
        computation = Computation.new().add_input_object([test_obj, test_obj])
        self.assertEqual(len(computation._input_cids), 2, "Adding List[obj] error")
        self.assertEqual(computation._input_cids, [correct_cid, correct_cid])


if __name__ == "__main__":
    unittest.main()
