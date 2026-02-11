import os
import shutil
import unittest
from pathlib import Path

import toml

from tests import get_config, get_config_dir, setup_sdk

config_path = Path(get_config_dir(), "config.toml")


class TestConfig(unittest.TestCase):
    """Tests config functions."""

    def test_01_default_config(self):
        if os.path.exists(get_config_dir()):
            shutil.rmtree(get_config_dir())
        self.assertFalse(os.path.exists(get_config_dir()), "Config directory already exists.")

        setup_sdk()
        self.assertTrue(os.path.exists(get_config_dir()), "Failed to create config directory.")
        # Config file is created on first setter call, not on init
        # Check that the directory exists

    def test_02_url(self):
        url = "http://www.example.com"
        get_config().set_integrity_service_url(url)

        with open(config_path, "r") as f:
            data = toml.load(f)

        self.assertEqual(data["url"], url, "Failed to save url to settings file")

    def test_03_store_all_blobs(self):
        get_config().set_store_all_blobs(True)

        with open(config_path, "r") as f:
            data = toml.load(f)

        self.assertEqual(
            data["store_all_blobs"], True, "Failed to save store_all_blobs to settings file"
        )

        get_config().set_store_all_blobs(False)

        with open(config_path, "r") as f:
            data = toml.load(f)

        self.assertEqual(
            data["store_all_blobs"], False, "Failed to save store_all_blobs to settings file"
        )

    def test_04_cid_ignore(self):
        get_config().set_cid_ignore_rules(True, False, False)

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
