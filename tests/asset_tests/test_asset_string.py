import asyncio
import unittest

from eqty_sdk import Custom, Dataset
from tests import setup_sdk


class DummyClass:
    def __init__(self, value):
        self.value = value

    def get_attr_test(self):
        """Dummy method to test attribute access."""


class AssetStringObject(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()
        cls.test_object = "This is a string value."
        # This is computed based on the string value above via eqty-cli.
        # correct_cid = "bafkr4idrf264dtnpujsdkxsr7f4llrrsok6lvt7l7lj4xujrlvuscr7kw4"
        cls.correct_cid = "bafkr4ibltkzdd4hsopruaikgkc425wf6tgluh3js5wsv2bxr7cgkoy2bmq"

        cls.asset = Custom.from_object(cls.test_object, name="Test string")

    def test_file_asset_cid(self):
        async def _cid():
            cid = self.asset.cid
            self.assertEqual(
                cid,
                self.correct_cid,
                f"CID mismatch, expected: {self.correct_cid}, got: {cid}",
            )

        asyncio.run(_cid())

    def test_dict_hash(self):
        """Tests uses cases in Living Content.
        The Asset wraps a string that is used for multiple lookups in Dicts, and built in python functions.
        """
        choice = Custom.from_object("1")
        choices = {}
        choices[choice] = "get_attr_test"

        self.assertIn(choice, choices, "Selection not found")

    def test_dict_str(self):
        """Tests uses cases in Living Content.
        The Asset needs explicitly converted to a str.
        """
        choice = Custom.from_object("get_attr_test", "Custom")

        dummy_class = DummyClass("1")
        func = getattr(dummy_class, str(choice), None)
        self.assertIsNotNone(func)

    def test_asset_default_values(self):
        """Test that the Asset is created with the default properties."""
        asset = Dataset.from_object(self.test_object)
        self.assertIsNotNone(asset.name, "Name should be a random name")
        self.assertEqual(asset.cid, self.correct_cid, "CID mismatch")
        self.assertEqual(asset.asset_type, "Dataset", "Asset type mismatch")
        self.assertFalse(asset._skip_proof, "skip_proof should default to False")

        with self.assertRaises(
            AttributeError, msg="Accessing undefined metadata should raise an exception"
        ):
            getattr(asset, "description")


if __name__ == "__main__":
    unittest.main()
