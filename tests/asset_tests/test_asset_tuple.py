import asyncio
import unittest

from eqty_sdk import CID, Custom, Model
from tests import setup_sdk


class AssetTupleObject(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

        cls.test_object = [("tuple", "item1"), ("tuple", "item2"), ("tuple", "item3")]
        # This is computed based on the tuple value above via eqty-cli.
        cls.correct_cid = CID("urn:cid:bafkr4ifl2kggudypuaw65wzuwo64lvdiljjfsw72ka6gelprl3ilpml56e")

        cls.asset = Custom.from_object(cls.test_object, "custom", name="Test tuple")

    def test_file_asset_cid(self):
        async def _cid():
            cid = self.asset.cid
            self.assertEqual(
                cid,
                self.correct_cid,
                f"CID mismatch, expected: {self.correct_cid}, got: {cid}",
            )

        asyncio.run(_cid())

    def test_asset_default_values(self):
        """Test that the Asset is created with the default properties."""
        asset = Custom.from_object(self.test_object, "Custom")
        self.assertIsNotNone(asset.name, "Name should be a random name")
        self.assertEqual(asset.cid, self.correct_cid, "CID mismatch")
        self.assertEqual(asset.asset_type, "Custom", "Asset type mismatch")
        self.assertFalse(asset._skip_proof, "skip_proof should default to False")

        with self.assertRaises(
            AttributeError, msg="Accessing undefined metadata should raise an exception"
        ):
            getattr(asset, "description")

    def test_asset_values(self):
        """Test that the Asset properties are set correctly."""
        name = "custom name"
        description = "custom description"
        asset = Model.from_object(
            self.test_object,
            name=name,
            description=description,
            skip_proof=True,
        )

        self.assertEqual(
            asset, self.test_object, "Asset should return the wrapped object by default"
        )
        self.assertEqual(
            asset.value, self.test_object, "Explicit call to access wrapped object failed"
        )

        self.assertEqual(asset.name, name, "Name not set")
        self.assertEqual(asset.description, description, "Description Failure")
        self.assertEqual(asset.asset_type, "Model", "Asset type mismatch")
        self.assertTrue(asset._skip_proof, "skip_proof override failed")


if __name__ == "__main__":
    unittest.main()
