import unittest
import uuid

from eqty_sdk import Context
from tests import setup_sdk


class TestContextFactory(unittest.TestCase):
    """Checks that the context changes work with the feature flag."""

    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_default_graphid_constructor(self):
        ctx = Context.new("test_default_graphid_constructor")

        if isinstance(ctx, Context):
            self.assertIsNotNone(ctx.name)
            self.assertIsNotNone(ctx.id)
            self.assertIsInstance(ctx.id, uuid.UUID)
            self.assertIsNone(ctx.parent)

    def test_graphid_with_parent(self):
        parent = Context.new("parent")
        ctx = Context.from_parent(parent).new("test_graphid_with_parent")
        self.assertIsNotNone(ctx.name)
        self.assertIsNotNone(ctx.id)
        self.assertIsInstance(ctx.id, uuid.UUID)
        self.assertEqual(ctx.name, "test_graphid_with_parent")
        self.assertEqual(ctx.parent, parent.id)

    def test_graphid_with_name(self):
        name = "Custom Name"
        ctx = Context.new(name)

        self.assertEqual(ctx.name, name)
        self.assertIsNotNone(ctx.id)
        self.assertIsInstance(ctx.id, uuid.UUID)
        self.assertIsNone(ctx.parent)


if __name__ == "__main__":
    unittest.main()
