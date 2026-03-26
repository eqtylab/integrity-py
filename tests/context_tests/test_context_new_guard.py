import unittest
import uuid

from eqty_sdk import Context
from tests import setup_sdk


class TestContextNewGuard(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_context_new_is_allowed_on_class(self):
        ctx = Context.new("root")
        self.assertEqual(ctx.name, "root")

    def test_context_new_raises_on_instance(self):
        gov_ctx = Context.from_uuid(uuid.UUID("00000000-0000-0000-0000-000000000001"))

        with self.assertRaises(TypeError):
            gov_ctx.new("child")


if __name__ == "__main__":
    unittest.main()
