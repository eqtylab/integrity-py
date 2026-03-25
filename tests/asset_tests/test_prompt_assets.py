import unittest
from pathlib import Path

from eqty_sdk import CID
from eqty_sdk.asset import Prompt, SystemPrompt
from tests import setup_sdk


class PromptAssetTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()
        cls.test_path = Path("tests/fixtures/assets/datasets/file/file_text.txt")
        cls.test_cid = CID("bafkr4ic6sphckk3a5x2fmgdeqod6tvv6k253q37icpfsk73wp2f524pok2")

    def test_prompt_from_object(self):
        asset = Prompt.from_object("Prompt body")
        self.assertEqual(asset.asset_type, "Prompt")
        self.assertEqual(str(asset), "Prompt body")

    def test_system_prompt_from_object(self):
        asset = SystemPrompt.from_object("System prompt body")
        self.assertEqual(asset.asset_type, "System_Prompt")
        self.assertEqual(str(asset), "System prompt body")

    def test_prompt_and_system_prompt_from_path(self):
        prompt = Prompt.from_path(self.test_path)
        system_prompt = SystemPrompt.from_path(self.test_path)

        self.assertEqual(prompt.asset_type, "Prompt")
        self.assertEqual(system_prompt.asset_type, "System_Prompt")
        self.assertEqual(prompt.value, self.test_path.resolve())
        self.assertEqual(system_prompt.value, self.test_path.resolve())

    def test_prompt_and_system_prompt_from_cid(self):
        prompt = Prompt.from_cid(self.test_cid)
        system_prompt = SystemPrompt.from_cid(self.test_cid)

        self.assertEqual(prompt.asset_type, "Prompt")
        self.assertEqual(system_prompt.asset_type, "System_Prompt")
        self.assertEqual(prompt.cid, self.test_cid)
        self.assertEqual(system_prompt.cid, self.test_cid)


if __name__ == "__main__":
    unittest.main()
