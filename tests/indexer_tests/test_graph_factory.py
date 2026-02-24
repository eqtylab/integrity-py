import sqlite3
import tempfile
import unittest
import uuid

from eqty_sdk import Context
from tests.rust import core_init, enable_logging


class GraphFactoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.logger = enable_logging(True)
        cls.temp_dir = tempfile.mkdtemp(prefix=f"{cls.__name__}_")
        core_init(cls.temp_dir)

    @classmethod
    def tearDownClass(cls):
        import shutil

        shutil.rmtree(cls.temp_dir)

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

    def test_context_from_project(self):
        project_id = uuid.UUID("00000000-0000-0000-0000-000000000001")
        child = Context.from_project(project_id).new("step-1")
        self.assertEqual(child.name, "step-1")
        self.assertEqual(child.parent, project_id)
        project_row = self._get_graph_row(project_id)
        child_row = self._get_graph_row(child.id)
        self.assertIsNotNone(project_row)
        self.assertEqual(project_row["parent_id"], None)
        self.assertIsNotNone(child_row)
        self.assertEqual(child_row["parent_id"], str(project_id))

    def _get_graph_row(self, graph_id):
        db_path = f\"{self.temp_dir}/graphs.db\"
        with sqlite3.connect(db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.cursor()
            cursor.execute(
                \"SELECT graph_id, name, parent_id FROM graphs WHERE graph_id = ?\",
                (str(graph_id),),
            )
            return cursor.fetchone()


if __name__ == "__main__":
    unittest.main()
