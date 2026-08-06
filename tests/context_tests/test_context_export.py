import json
import os
import tempfile
import unittest
from pathlib import Path

from eqty_sdk import Context, Signer, set_active_signer
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

    @unittest.skipUnless(
        os.getenv("EQTY_TEST_VCOMP", "").lower() in ("true", "1", "yes", "on"),
        "EQTY_TEST_VCOMP not enabled",
    )
    def test_export_folds_notary_did_registration(self):
        """With a VComp notary signer active, export embeds its DID registration.

        A fresh context has no computation, so retrieve_statements returns
        nothing; before the fold the manifest was empty and failed verify_did
        standalone. The signer's cached did_statements/did_blobs must be folded in.
        """
        signer = Signer.vcomp_notary("http://localhost:8066")
        set_active_signer(signer)
        ctx = Context.new("export-notary-did")
        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / "manifest.json"
            ctx.export(dest)
            manifest = json.loads(dest.read_text())

        self.assertTrue(manifest["statements"])
        self.assertTrue(manifest["blobs"])


if __name__ == "__main__":
    unittest.main()
