import logging
import unittest

from eqty_sdk import config
from eqty_sdk._rust import Graph
from tests import setup_sdk


class TestInitContext(unittest.TestCase):
    """Checks that the Initialization of the sdk handles context properly."""

    logger = logging.getLogger(__name__)
    logger.setLevel(logging.DEBUG)

    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_init_context(self):
        """Test that init creates a default root context."""
        ctx = config.root_context()

        self.assertIsNotNone(ctx)
        self.assertIsInstance(ctx, Graph)
        self.assertIsNotNone(ctx.id)

    def test_set_default_graph(self):
        """Test that set_default_graph updates the root context."""
        new_graph = Graph.new()
        config.set_default_graph(new_graph)

        ctx = config.root_context()
        self.assertEqual(ctx.id, new_graph.id)


if __name__ == "__main__":
    unittest.main()
