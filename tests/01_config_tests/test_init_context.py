import logging
import unittest

from eqty_sdk import init
from eqty_sdk._rust import Config, Graph
from tests import setup_sdk


class TestInitContext(unittest.TestCase):
    """Checks that the Initialization of the sdk handles context properly."""

    logger = logging.getLogger(__name__)
    logger.setLevel(logging.DEBUG)

    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_init_context(self):
        """Test that init returns a Config instance."""
        cfg = init()
        self.assertIsNotNone(cfg)
        self.assertIsInstance(cfg, Config)

    def test_set_default_graph(self):
        """Test that set_default_graph can be called."""
        cfg = init()
        new_graph = Graph.new()
        cfg.set_default_graph(new_graph)


if __name__ == "__main__":
    unittest.main()
