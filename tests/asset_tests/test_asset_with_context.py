import sqlite3
import unittest

from eqty_sdk import Context, Dataset
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


class TestAssetWithContext(unittest.TestCase):
    def test_with_context_uses_graph(self):
        setup_sdk()

        ctx = Context.new("ctx")
        Dataset.with_context(ctx).from_object(123, store=False, name="From Object")

        db_path = f"{get_config_dir()}/graphs.db"
        link_count = _count_statement_links(db_path, str(ctx.id))
        self.assertGreater(link_count, 0, "Expected statements linked to custom context")


if __name__ == "__main__":
    unittest.main()
