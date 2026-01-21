import shutil
import tempfile
import unittest
from pathlib import Path

import eqty_sdk._rust as core


class CidPath(unittest.TestCase):
    test_dir = "../fixtures/cid-compute/iroh-file"
    correct_dir_cid = "bagaachraoa227m2kqyuhhuesinadjyvmf2thwq6bp6rbkdyjiitechi5ip6a"

    test_file = "../fixtures/cid-compute/iroh-file/example.txt"
    correct_file_cid = Path(f"{test_file}.cid").read_text().strip()

    @classmethod
    def setUpClass(cls):
        cls.temp_dir = tempfile.mkdtemp(prefix=f"{cls.__name__}_")
        core.context.init(cls.temp_dir)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.temp_dir)

    def test_file_cid(self):
        result = core.cid.compute_cid_for_file(self.test_file)
        self.assertEqual(result.cid, self.correct_file_cid, "File CID mismatch")

    def test_dir_cid(self):
        result = core.cid.compute_cid_for_directory(self.test_dir)
        self.assertEqual(result.collection.cid, self.correct_dir_cid, "Dir collection CID mismatch")
        self.assertEqual(
            result.meta.cid,
            "bafkr4ia3gwhaokl3f7k62ec23x7vww22nhwlynksavrjv7idrgc7dhnif4",
            "Dir meta CID mismatch",
        )

    def test_bad_dir(self):
        with self.assertRaises(RuntimeError):
            core.cid.compute_cid_for_file("Not A DIR")


if __name__ == "__main__":
    unittest.main()
