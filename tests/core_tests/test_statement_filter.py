import os
import shutil
import unittest

from eqty_sdk import Dataset, Statements
from tests import setup_sdk

# When VCs are enabled
# num_statements_per_dataset = 4  # 1 DataRegistration, 1 MetadataRegistration, 2 VCs
# When VCs are disabled
num_statements_per_dataset = 2  # 1 DataRegistration, 1 MetadataRegistration


class TestStatementFilter(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # enable_logging(True)

        if os.path.exists("tmp/TestStatementFilter"):
            shutil.rmtree("tmp/TestStatementFilter")

        setup_sdk()

    def test_simple(self):
        """Test simple."""
        Statements().delete_all()

        d0 = Dataset.from_object("d0", store=True, name="d0")
        d0.add_attribute(session="session_0")

        d1 = Dataset.from_object("d1", store=True, name="d1")
        d1.add_attribute(session="session_1")

        statements = Statements().select("attributes.session == 'session_0'")
        # there should 4 statements, 1 DataRegistration, 1 MetadataRegistration, 2 VCs
        self.assertEqual(len(statements.statements), 1 * num_statements_per_dataset)

        statements = Statements().select("attributes.session == 'session_1'")
        self.assertEqual(len(statements.statements), 1 * num_statements_per_dataset)

        statements = Statements().select(
            "attributes.session == 'session_0' || attributes.session == 'session_1'"
        )
        self.assertEqual(len(statements.statements), 2 * num_statements_per_dataset)

        statements = Statements().select_all()
        self.assertEqual(len(statements.statements), 2 * num_statements_per_dataset)

    def test_complex(self):
        """Test complex."""
        Statements().delete_all()

        d0 = Dataset.from_object("d0", store=True, name="d0")
        d0.add_attribute(session="session_0", user="user_0")

        d1 = Dataset.from_object("d1", store=True, name="d1")
        d1.add_attribute(session="session_1", user="user_1")

        d2 = Dataset.from_object("d2", store=True, name="d2")
        d2.add_attribute(session="session_0", user="user_1")

        statements = Statements().select(
            "attributes.session == 'session_0' && attributes.user == 'user_0'"
        )
        # there should 4 statements, 1 DataRegistration, 1 MetadataRegistration, 2 VCs
        self.assertEqual(len(statements.statements), 1 * num_statements_per_dataset)

        statements = Statements().select(
            "attributes.session == 'session_0' && attributes.user == 'user_1'"
        )
        self.assertEqual(len(statements.statements), 1 * num_statements_per_dataset)

        statements = Statements().select(
            "attributes.session == 'session_0' || attributes.user == 'user_0'"
        )
        self.assertEqual(len(statements.statements), 2 * num_statements_per_dataset)

        statements = Statements().select(
            "attributes.session == 'session_0' || attributes.user == 'user_1'"
        )
        self.assertEqual(len(statements.statements), 3 * num_statements_per_dataset)

        statements = Statements().select_all()
        self.assertEqual(len(statements.statements), 3 * num_statements_per_dataset)

    def test_range(self):
        """Test range."""
        Statements().delete_all()

        d0 = Dataset.from_object("d0", store=True, name="d0")
        d0.add_attribute(value=5)

        d1 = Dataset.from_object("d1", store=True, name="d1")
        d1.add_attribute(value=10)

        d2 = Dataset.from_object("d2", store=True, name="d2")
        d2.add_attribute(value=15)

        statements = Statements().select("attributes.value < 10")
        # there should 4 statements, 1 DataRegistration, 1 MetadataRegistration, 2 VCs
        self.assertEqual(len(statements.statements), 1 * num_statements_per_dataset)

        statements = Statements().select("attributes.value < 11")
        self.assertEqual(len(statements.statements), 2 * num_statements_per_dataset)

        statements = Statements().select("attributes.value > 10")
        self.assertEqual(len(statements.statements), 1 * num_statements_per_dataset)

        statements = Statements().select("attributes.value > 9")
        self.assertEqual(len(statements.statements), 2 * num_statements_per_dataset)

        statements = Statements().select("attributes.value > 4 && attributes.value < 16")
        self.assertEqual(len(statements.statements), 3 * num_statements_per_dataset)

        statements = Statements().select_all()
        self.assertEqual(len(statements.statements), 3 * num_statements_per_dataset)


if __name__ == "__main__":
    unittest.main()
