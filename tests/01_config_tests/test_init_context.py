import logging
import unittest
from uuid import UUID, uuid4

from eqty_sdk import init
from eqty_sdk.config.config import Config
from eqty_sdk.context import GraphIDCtx
from eqty_sdk.feature_flags import FEATURE_FLAGS, FeatureFlags
from tests import setup_sdk


# @unittest.skip("testing")
class TestInitContext(unittest.TestCase):
    """Checks that the Initializtion of the sdk handles parent context."""

    logger = logging.getLogger(__name__)
    logger.setLevel(logging.DEBUG)
    config = None

    @classmethod
    def setUpTest(cls):
        cls.config = setup_sdk()

    def setUp(self):
        """Clear feature flags and reset config singleton before each test."""
        FeatureFlags.clear_runtime()
        # Reset the config singleton to allow fresh initialization

        Config._instance = None
        # Re-setup SDK for fresh test
        self.config = setup_sdk()

    def tearDown(self):
        Config._instance = self.config

    def test_init_context(self):
        """Test that init creates a default root context."""
        FeatureFlags.enable(FEATURE_FLAGS.GRAPH_IDS)
        config = init()

        self.assertIsNotNone(config.root_context)
        self.assertIsInstance(config.root_context, GraphIDCtx)
        self.assertIsNotNone(config.root_context.uuid)
        self.assertEqual(str(config.root_context.uuid), config.root_context.name)

    def test_init_with_context(self):
        """Test that with_context can set the root context."""
        FeatureFlags.enable(FEATURE_FLAGS.GRAPH_IDS)
        name = "Unit Test"
        ctx = UUID("12345678123456781234567812345678")
        self.logger.info(f"Parent Context: {ctx!r}")
        config = init().from_context(ctx, name)

        self.assertIsNotNone(config.root_context)
        self.assertIsInstance(config.root_context, GraphIDCtx)
        self.assertEqual(config.root_context.parent_ctx, ctx)
        self.assertEqual(config.root_context.name, name)
        self.assertIsNotNone(config.root_context.uuid)

    def test_id_reuse(self):
        """Test that init with_context can be set with an explicit UUID."""
        FeatureFlags.enable(FEATURE_FLAGS.GRAPH_IDS)
        name = "Unit Test"
        ctx = uuid4()

        config = init().with_context(name, ctx)

        self.assertEqual(config.root_context.uuid, ctx)
        self.assertEqual(config.root_context.name, name)


if __name__ == "__main__":
    unittest.main()
