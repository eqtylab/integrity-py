import os
import shutil
import unittest
from pathlib import Path

import toml

from eqty_sdk import Config
from tests import get_config_dir, setup_sdk

config_path = Path(get_config_dir(), "config.toml")


class TestConfig(unittest.TestCase):
    """Tests config functions."""

    def test_01_default_config(self):
        if os.path.exists(get_config_dir()):
            shutil.rmtree(get_config_dir())
        self.assertFalse(os.path.exists(get_config_dir()), "Config directory already exists.")

        setup_sdk()
        self.assertTrue(os.path.exists(get_config_dir()), "Failed to create config directory.")
        self.assertTrue(os.path.exists(config_path), "Failed to create config.toml")

    def test_02_url(self):
        config = Config()
        self.assertIsNone(config._settings.url)

        url = "http://www.example.com"
        config.set_integrity_service_url(url)
        self.assertEqual(config._settings.url, url)

        with open(config_path, "r") as f:
            data = toml.load(f)

        self.assertEqual(data["url"], url, "Failed to save url to settings file")

    def test_03_store_all_blobs(self):
        config = Config()
        self.assertFalse(config._settings.store_all_blobs)

        config.set_store_all_blobs(True)
        self.assertTrue(config._settings.store_all_blobs)

        with open(config_path, "r") as f:
            data = toml.load(f)

        self.assertEqual(
            data["store_all_blobs"], True, "Failed to save store_all_blobs to settings file"
        )

        config.set_store_all_blobs(False)
        self.assertFalse(config._settings.store_all_blobs)

        with open(config_path, "r") as f:
            data = toml.load(f)

        self.assertEqual(
            data["store_all_blobs"], False, "Failed to save store_all_blobs to settings file"
        )

    def test_04_cid_ignore(self):
        config = Config()
        self.assertFalse(config._settings.cid_ignore.include_hidden_files)
        self.assertFalse(config._settings.cid_ignore.gitignore)
        self.assertFalse(config._settings.cid_ignore.include_symlinks)

        config.set_cid_ignore(True, False, False)

        self.assertTrue(config._settings.cid_ignore.include_hidden_files)
        self.assertFalse(config._settings.cid_ignore.gitignore)
        self.assertFalse(config._settings.cid_ignore.include_symlinks)

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


if __name__ == "__main__":
    unittest.main()
