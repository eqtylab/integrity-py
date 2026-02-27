import sqlite3
import unittest
import uuid

from eqty_sdk import Context
from tests import get_config_dir, setup_sdk


class GraphFactoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cfg = setup_sdk()

    def test_context_new(self):
        ctx = Context.new("root")
        self.assertEqual(ctx.name, "root")
        self.assertIsNotNone(ctx.id)
        self.assertIsNone(ctx.parent)
        row = self._get_graph_row(ctx.id)
        self.assertIsNotNone(row)
        self.assertEqual(row["name"], "root")
        self.assertIsNone(row["parent_id"])

    def test_context_from_parent(self):
        parent = Context.new("parent")
        child = Context.from_parent(parent).new("child")
        self.assertEqual(child.name, "child")
        self.assertEqual(child.parent, parent.id)
        parent_row = self._get_graph_row(parent.id)
        child_row = self._get_graph_row(child.id)
        self.assertIsNotNone(parent_row)
        self.assertIsNotNone(child_row)
        self.assertEqual(child_row["parent_id"], str(parent.id))

    def test_context_from_uuid(self):
        project_id = uuid.UUID("00000000-0000-0000-0000-000000000001")
        child = Context.from_uuid(project_id).new("step-1")
        self.assertEqual(child.name, "step-1")
        self.assertEqual(child.parent, project_id)
        project_row = self._get_graph_row(project_id)
        child_row = self._get_graph_row(child.id)
        self.assertIsNotNone(project_row)
        self.assertEqual(project_row["parent_id"], None)
        self.assertIsNotNone(child_row)
        self.assertEqual(child_row["parent_id"], str(project_id))

    def _get_graph_row(self, graph_id):
        db_path = f"{get_config_dir()}/graphs.db"
        with sqlite3.connect(db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.cursor()
            cursor.execute(
                "SELECT graph_id, name, parent_id FROM graphs WHERE graph_id = ?",
                (str(graph_id),),
            )
            return cursor.fetchone()


if __name__ == "__main__":
    unittest.main()
