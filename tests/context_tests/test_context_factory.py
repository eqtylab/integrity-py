import unittest
import uuid

from eqty_sdk.context import Context, GraphIDCtx, OriginalCtx
from eqty_sdk.feature_flags import FEATURE_FLAGS, FeatureFlags
from tests import setup_sdk


class TestContextFactory(unittest.TestCase):
    """Checks that the context changes work with the feature flag."""

    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def setUp(self):
        """Clear feature flags before each test."""
        FeatureFlags.clear_runtime()

    def test_context_creation(self):
        """Test Context factory creates correct type based on feature flag."""
        project_id = str(uuid.uuid4())

        FeatureFlags.disable(FEATURE_FLAGS.GRAPH_IDS)
        ctx = Context(project_id=project_id)

        self.assertIsInstance(ctx, OriginalCtx)
        if isinstance(ctx, OriginalCtx):
            self.assertEqual(ctx.project_id, project_id)

    def test_default_graphid_constructor(self):
        FeatureFlags.enable(FEATURE_FLAGS.GRAPH_IDS)
        ctx = Context()

        self.assertIsInstance(ctx, GraphIDCtx)
        if isinstance(ctx, GraphIDCtx):
            self.assertIsNotNone(ctx.name)
            self.assertIsNotNone(ctx.uuid)
            self.assertIsInstance(ctx.uuid, uuid.UUID)
            self.assertIsNone(ctx.parent_ctx)

    def test_graphid_with_parent(self):
        FeatureFlags.enable(FEATURE_FLAGS.GRAPH_IDS)
        id = uuid.uuid4()
        ctx = Context(id=id)
        parent = uuid.uuid4()

        self.assertIsInstance(ctx, GraphIDCtx)
        if isinstance(ctx, GraphIDCtx):
            ctx.parent_ctx = parent
            self.assertIsNotNone(ctx.name)
            self.assertIsNotNone(ctx.uuid)
            self.assertIsInstance(ctx.uuid, uuid.UUID)
            self.assertEqual(ctx.name, str(id))
            self.assertEqual(ctx.parent_ctx, parent)

    def test_graphid_with_name(self):
        FeatureFlags.enable(FEATURE_FLAGS.GRAPH_IDS)
        name = "Custom Name"
        ctx = Context(name=name)

        self.assertIsInstance(ctx, GraphIDCtx)
        if isinstance(ctx, GraphIDCtx):
            self.assertEqual(ctx.name, name)
            self.assertIsNotNone(ctx.uuid)
            self.assertIsInstance(ctx.uuid, uuid.UUID)
            self.assertIsNone(ctx.parent_ctx)


if __name__ == "__main__":
    unittest.main()
