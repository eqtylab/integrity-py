import tempfile
import unittest
from pathlib import Path

from eqty_sdk.asset.asset import serialize_for_hashing


class Dummy:
    def __init__(self):
        self.value = 42


class DummyModel:
    def state_dict(self):
        return {"weights": [1, 2, 3]}


class DummyHasModel:
    def __init__(self):
        self.model = DummyModel()


class TestSerializeForHashing(unittest.TestCase):
    def test_pathlike(self):
        temp_dir = tempfile.mkdtemp(prefix="asset_hash_")
        p = Path(temp_dir) / "file.txt"
        p.write_text("abc", encoding="utf-8")
        result = serialize_for_hashing(p)
        self.assertIsInstance(result, (bytes, bytearray))

    def test_model_state_dict(self):
        obj = DummyHasModel()
        result = serialize_for_hashing(obj)
        self.assertIsInstance(result, (bytes, bytearray))

    def test_dict_fallback(self):
        obj = Dummy()
        result = serialize_for_hashing(obj)
        self.assertIsInstance(result, (bytes, bytearray))


if __name__ == "__main__":
    unittest.main()
