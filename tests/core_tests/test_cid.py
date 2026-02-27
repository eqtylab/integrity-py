import os
import unittest
from pathlib import Path

from eqty_sdk._rust import (
    get_cid_for_bytes,
    get_cid_for_path,
)
from tests import get_config_dir, setup_sdk


class TestCoreCid(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cfg = setup_sdk()

    def test_bytes_cid_calc(self):
        test_bytes = b"hello world"
        correct_cid = "bafkr4igxjga67jykbseaxdmmdgc5a5o3zp3htom2l6mrjznk7fvyggu6eq"

        cid = get_cid_for_bytes(test_bytes)
        self.assertEqual(cid, correct_cid)

    def test_dir_cid(self):
        test_dir = Path("./tests/fixtures/iroh/collection").resolve()
        cid = get_cid_for_path(test_dir, store=True)

        self.assertEqual(cid, "bagaachraq547k4actjefuc4u2t5ait2c2ozfgs2euayoujog4lzn7khy2b6a")
        blob_dir = get_config_dir().joinpath("blobs")
        iroh_collection = blob_dir.joinpath(
            "bagaachraq547k4actjefuc4u2t5ait2c2ozfgs2euayoujog4lzn7khy2b6a"
        )
        self.assertTrue(os.path.exists(iroh_collection), "iroh_collection not saved")

        self.cfg.set_cid_ignore_rules(include_symlinks=True)

        symlink_src = Path("./tests/fixtures/assets/datasets/file/file_text.txt").resolve()
        symlink_dst = Path("./tests/fixtures/iroh/collection/linked_file.txt").resolve()
        os.symlink(symlink_src, symlink_dst)

        cid = get_cid_for_path(test_dir)
        self.assertEqual(cid, "bagaachrarggs2jlg2y6fpoe63u7m46lu7whz57ifauwsnyuzj33o6phffkcq")
        os.unlink(symlink_dst)

    def test_iroh_collections(self):
        test_dir = Path("./tests/fixtures/iroh/collection").resolve()
        cid = get_cid_for_path(test_dir, store=True)
        self.assertEqual(
            "bagaachraq547k4actjefuc4u2t5ait2c2ozfgs2euayoujog4lzn7khy2b6a",
            cid,
            "Collection CID mismatch",
        )

        blob_dir = get_config_dir().joinpath("blobs")
        iroh_collection = blob_dir.joinpath(
            "bagaachraq547k4actjefuc4u2t5ait2c2ozfgs2euayoujog4lzn7khy2b6a"
        )
        self.assertTrue(os.path.exists(iroh_collection), "iroh_collection not saved")

        iroh_metadata = blob_dir.joinpath(
            "bafkr4ibjcxelwo3leme7bbacs54mjfixjffxb6zmezpbvw63yx5ujpe7ku"
        )
        self.assertTrue(os.path.exists(iroh_metadata), "iroh_collection metadata not saved")

        file1_blob = blob_dir.joinpath(
            "bafkr4ihq7zatrjwxohiag57v63nclpksnzh3kahjuywqfxqtpl4yikhnje"
        )
        self.assertTrue(os.path.exists(file1_blob), "file1 blob not saved")

        file2_blob = blob_dir.joinpath(
            "bafkr4iedszw5ke2ylp4336ojwqmpmg4tbscqq6cbrckxd5piwe3hgxt7wa"
        )
        self.assertTrue(os.path.exists(file2_blob), "file2 blob not saved")


if __name__ == "__main__":
    unittest.main()
