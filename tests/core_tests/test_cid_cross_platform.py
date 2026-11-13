import tempfile
import unittest
from pathlib import Path

from eqty_sdk._rust import CID, get_cid_for_path
from tests import setup_sdk

FIXTURES = Path("tests/fixtures")

# Literals, not read from fixtures: every OS must produce exactly these.
KNOWN_CIDS = {
    "assets/datasets/file/file_text.txt": "bafkr4igdry6c65svzqkty7sa3jzcwrrijtbrfcikvp4usedd6bcgg7mmi4",
    "iroh/collection": "bagaachraq547k4actjefuc4u2t5ait2c2ozfgs2euayoujog4lzn7khy2b6a",
}


class TestCidCrossPlatform(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_known_cids(self):
        for rel, expected in KNOWN_CIDS.items():
            with self.subTest(path=rel):
                self.assertEqual(get_cid_for_path(FIXTURES.joinpath(rel).resolve()), CID(expected))

    def test_known_cid_for_directory_with_subdirectory(self):
        r"""Fails if a platform names collection entries with '\' instead of '/'."""
        # Not under tests/fixtures: the model-signing example CIDs that whole tree.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            root.joinpath("sub").mkdir()
            root.joinpath("top.txt").write_bytes(b"top\n")
            root.joinpath("sub", "inner.txt").write_bytes(b"inner\n")
            self.assertEqual(
                get_cid_for_path(root.resolve()),
                CID("bagaachraajlssptwr3otrkcma53y4o24heyywrcngrhb3e56syihkkla3pnq"),
            )


if __name__ == "__main__":
    unittest.main()
