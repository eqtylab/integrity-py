import sqlite3
import unittest

from eqty_sdk import Dataset
from tests import get_config_dir, setup_sdk


def _count_data_statements(db_path: str) -> int:
    with sqlite3.connect(db_path) as conn:
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) FROM data_statements")
        row = cursor.fetchone()
        return int(row[0]) if row else 0


def _count_metadata_statements(db_path: str) -> int:
    with sqlite3.connect(db_path) as conn:
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) FROM metadata_statements")
        row = cursor.fetchone()
        return int(row[0]) if row else 0


class TestAssetSkipRegistration(unittest.TestCase):
    def test_skip_registration(self):
        setup_sdk()

        db_path = f"{get_config_dir()}/graphs.db"
        before_data = _count_data_statements(db_path)
        before_meta = _count_metadata_statements(db_path)

        Dataset.from_object("data", store=False, name="skip", skip_registration=True)

        self.assertEqual(_count_data_statements(db_path), before_data)
        self.assertEqual(_count_metadata_statements(db_path), before_meta)


if __name__ == "__main__":
    unittest.main()
