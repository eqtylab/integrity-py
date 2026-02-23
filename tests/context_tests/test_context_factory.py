import unittest
import uuid

from eqty_sdk.context import Context

from tests import setup_sdk


class TestContextFactory(unittest.TestCase):
    """Checks that the context changes work with the feature flag."""

    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_default_graphid_constructor(self):
        ctx = Context()

        if isinstance(ctx, Context):
            self.assertIsNotNone(ctx.name)
            self.assertIsNotNone(ctx.uuid)
            self.assertIsInstance(ctx.uuid, uuid.UUID)
            self.assertIsNone(ctx.parent_ctx)

    def test_graphid_with_parent(self):
        id = uuid.uuid4()
        ctx = Context(id=id)
        parent = uuid.uuid4()

        ctx.parent_ctx = parent
        self.assertIsNotNone(ctx.name)
        self.assertIsNotNone(ctx.uuid)
        self.assertIsInstance(ctx.uuid, uuid.UUID)
        self.assertEqual(ctx.name, str(id))
        self.assertEqual(ctx.parent_ctx, parent)

    def test_graphid_with_name(self):
        name = "Custom Name"
        ctx = Context(name=name)

        self.assertEqual(ctx.name, name)
        self.assertIsNotNone(ctx.uuid)
        self.assertIsInstance(ctx.uuid, uuid.UUID)
        self.assertIsNone(ctx.parent_ctx)


if __name__ == "__main__":
    unittest.main()
