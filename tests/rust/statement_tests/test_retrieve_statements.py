import tempfile
import unittest

from eqty_sdk._rust import statements
from tests.rust import core_init, enable_logging


class StatementTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        """Set up test fixtures once for the entire test class."""
        cls.logger = enable_logging(True)
        cls.temp_dir = tempfile.mkdtemp(prefix=f"{cls.__name__}_")
        core_init(cls.temp_dir)

    @classmethod
    def tearDownClass(cls):
        """Clean up test fixtures after all tests are done."""
        import shutil

        shutil.rmtree(cls.temp_dir)

    def test_retrieve_statements(self):
        timestamp = "2025-08-26T14:53:29Z"
        input_id = statements.create_data_statement(["urn:cid:input1"], timestamp)
        statements.add_attributes_to_statements(
            [input_id], {"type": "input", "test name": "test_retrieve_statements"}
        )
        output_id = statements.create_data_statement(["urn:cid:output1"], timestamp)
        statements.add_attributes_to_statements(
            [output_id], {"type": "output", "test name": "test_retrieve_statements"}
        )
        statement_id = statements.create_computation_statement(
            [input_id], [output_id], None, None, None, timestamp
        )
        self.assertEqual(
            "urn:cid:bagb6qaq6ebko7ylmijmmedmv3y62iq7az5akibw56cl4r7hyb2xqcvrmbnnmi",
            statement_id,
            "Error creating test statement",
        )
        attributes = {"test name": "test_retrieve_statements"}
        statements.add_attributes_to_statements([statement_id], attributes)

        (_, a) = statements.retrieve_statements(
            "attributes.\"test name\" == 'test_retrieve_statements'"
        )
        self.logger.info("ATTRIBUTES %s", a)
        self.assertEqual(attributes, a[statement_id])
        self.assertEqual({"type": "input", "test name": "test_retrieve_statements"}, a[input_id])
        pass


if __name__ == "__main__":
    unittest.main()
