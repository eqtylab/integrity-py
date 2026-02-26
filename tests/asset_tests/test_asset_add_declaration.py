import sqlite3
import unittest

from eqty_sdk import Dataset, Declaration
from tests import get_config_dir, setup_sdk


def _count_governance(db_path: str) -> int:
    with sqlite3.connect(db_path) as conn:
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) FROM governance_statements")
        row = cursor.fetchone()
        return int(row[0]) if row else 0


class TestAssetAddDeclaration(unittest.TestCase):
    def test_add_declaration_creates_governance_statement(self):
        setup_sdk()

        asset = Dataset.from_object("data", store=False, name="decl")
        declaration = Declaration.new("subject", "statement")
        asset.add_declaration(declaration)

        db_path = f"{get_config_dir()}/graphs.db"
        self.assertEqual(_count_governance(db_path), 1)


if __name__ == "__main__":
    unittest.main()
