import os
import tempfile
import unittest
from pathlib import Path

from eqty_sdk import Context
from tests import setup_sdk


class ContextExportTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cfg = setup_sdk()

    def test_export_creates_missing_parent_dirs(self):
        """export() should create the destination directory.

        Every example in examples/ writes to ./manifests/<name>.json, a directory
        that does not exist in a fresh project. Before this was fixed, export()
        called File::create() directly and raised "Failed to create manifest file:
        No such file or directory".
        """
        ctx = Context.new("export-nested")
        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / "manifests" / "nested" / "manifest.json"
            self.assertFalse(dest.parent.exists())

            ctx.export(dest)

            self.assertTrue(dest.is_file())
            self.assertGreater(dest.stat().st_size, 0)

    def test_export_to_bare_filename_in_cwd(self):
        """A bare filename has an empty parent; that must not be treated as a dir."""
        ctx = Context.new("export-bare")
        with tempfile.TemporaryDirectory() as tmp:
            prev = os.getcwd()
            os.chdir(tmp)
            try:
                ctx.export(Path("manifest.json"))
                self.assertTrue(Path("manifest.json").is_file())
            finally:
                os.chdir(prev)


if __name__ == "__main__":
    unittest.main()
