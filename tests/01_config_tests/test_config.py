import os
import shutil
import unittest
from pathlib import Path

import toml

from eqty_sdk import init
from tests import get_config_dir

config_path = Path(get_config_dir(), "config.toml")


class TestConfig(unittest.TestCase):
    """Tests config functions."""

    def test_01_default_config(self):
        if os.path.exists(get_config_dir()):
            shutil.rmtree(get_config_dir())
        self.assertFalse(os.path.exists(get_config_dir()), "Config directory already exists.")

        init(custom_dir=get_config_dir())
        self.assertTrue(os.path.exists(get_config_dir()), "Failed to create config directory.")
        # Config file is created on first setter call, not on init
        # Check that the directory exists

    def test_02_hashing_config(self):
        cfg = init(custom_dir=get_config_dir())
        self.assertIsNotNone(cfg.set_hashing_config(multithread=True, memory_map=True))

    def test_03_store_all_blobs(self):
        cfg = init(custom_dir=get_config_dir())
        cfg.set_store_all_blobs(True)

        with open(config_path, "r") as f:
            data = toml.load(f)

        self.assertEqual(
            data["store_all_blobs"], True, "Failed to save store_all_blobs to settings file"
        )

        cfg.set_store_all_blobs(False)

        with open(config_path, "r") as f:
            data = toml.load(f)

        self.assertEqual(
            data["store_all_blobs"], False, "Failed to save store_all_blobs to settings file"
        )

    def test_04_cid_ignore(self):
        cfg = init(custom_dir=get_config_dir())
        cfg.set_cid_ignore_rules(True, False, False)

        with open(config_path, "r") as f:
            data = toml.load(f)

        self.assertEqual(
            data["cid_ignore"]["include_hidden_files"],
            True,
            "Failed to save cid_ignore to settings file",
        )
        self.assertEqual(
            data["cid_ignore"]["gitignore"], False, "Failed to save cid_ignore to settings file"
        )
        self.assertEqual(
            data["cid_ignore"]["include_symlinks"],
            False,
            "Failed to save cid_ignore to settings file",
        )

    def test_05_cid_ignore_symlink_behavior(self):
        cfg = init(custom_dir=get_config_dir())
        cfg.set_cid_ignore_rules(include_symlinks=True)

        test_dir = Path("./tests/fixtures/iroh/collection").resolve()
        symlink_src = Path("./tests/fixtures/assets/datasets/file/file_text.txt").resolve()
        symlink_dst = Path("./tests/fixtures/iroh/collection/linked_file.txt").resolve()
        if os.path.lexists(symlink_dst):
            symlink_dst.unlink()

        try:
            os.symlink(symlink_src, symlink_dst)

            from eqty_sdk import get_cid_for_path

            cid = get_cid_for_path(test_dir, _store=True)
            self.assertEqual(
                str(cid), "urn:cid:bagaachrarggs2jlg2y6fpoe63u7m46lu7whz57ifauwsnyuzj33o6phffkcq"
            )
        finally:
            if os.path.lexists(symlink_dst):
                symlink_dst.unlink()


if __name__ == "__main__":
    unittest.main()
