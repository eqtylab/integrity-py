import sqlite3
import unittest

from eqty_sdk import CID, Computation, Context, Dataset
from tests import get_config_dir, setup_sdk


def _count_statement_links(db_path: str, graph_id: str) -> int:
    with sqlite3.connect(db_path) as conn:
        cursor = conn.cursor()
        cursor.execute(
            "SELECT COUNT(*) FROM statement_graph_link WHERE graph_id = ?",
            (graph_id,),
        )
        row = cursor.fetchone()
        return int(row[0]) if row else 0


class TestContextLinking(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_dataset_with_context_links_statements(self):
        ctx = Context.new("ctx_dataset")
        Dataset.with_context(ctx).from_object("data", store=False, name="With Context")

        db_path = f"{get_config_dir()}/graphs.db"
        link_count = _count_statement_links(db_path, str(ctx.id))
        self.assertGreater(link_count, 0)

    def test_computation_with_context_links_statements(self):
        ctx = Context.new("ctx_comp")
        Computation.with_context(ctx).new(name="Test Computation").add_input_cid(
            CID("bafkreigh2akiscaildc3n6cnq3g5u5j4f6l5j5x4ux7z4x3t3j3t5v7szy")
        ).add_output_cid(
            CID("bafkreigh2akiscaildc3n6cnq3g5u5j4f6l5j5x4ux7z4x3t3j3t5v7szy")
        ).finalize()

        db_path = f"{get_config_dir()}/graphs.db"
        link_count = _count_statement_links(db_path, str(ctx.id))
        self.assertGreater(link_count, 0)

    def test_statements_linked_to_separate_contexts(self):
        ctx_a = Context.new("ctx_a")
        ctx_b = Context.new("ctx_b")

        Dataset.with_context(ctx_a).from_object("data-a", store=False, name="A")
        Dataset.with_context(ctx_b).from_object("data-b", store=False, name="B")

        db_path = f"{get_config_dir()}/graphs.db"
        links_a = _count_statement_links(db_path, str(ctx_a.id))
        links_b = _count_statement_links(db_path, str(ctx_b.id))

        self.assertGreater(links_a, 0)
        self.assertGreater(links_b, 0)


if __name__ == "__main__":
    unittest.main()
