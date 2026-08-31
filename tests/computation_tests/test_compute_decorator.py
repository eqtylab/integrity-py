import json
import logging
import unittest
from pathlib import Path

from eqty_sdk import Dataset, UsageError, compute, init, purge_blob_store
from tests import get_config_dir, setup_sdk

logger = logging.getLogger("unittests")

# CIDs for the literal strings used below, precomputed independently of any
# shared test state (calling get_cid_for_bytes at test time would "prime" the
# CID as already-known and mask whether the blob was actually stored).
NOT_RETAINED_CID = "bafkr4ig4ocetv6dfahrvws3k2f24xmqkcfcnghhc235leltmz74hwhnmam"
RETAIN_ME_CID = "bafkr4if5yhakzvf3itenakgmw34ahzpailts4p4jk6xptdwpwtwalieppe"
OVERRIDE_DEFAULT_CID = "bafkr4id4epbxd4vhayelzz2cj3nyvo54y2idg52w63tbimqejqcgng4kj4"


def load_mock_data(file_path):
    with open(file_path, "r") as file:
        data = json.load(file)
    return data


class TestComputeDecorator(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        logger.debug("Setting up test case.")
        cls.environment_url = "https://dev.integrity.dev.eqtylab.io"
        cls.project_name = cls.__name__

        setup_sdk()
        logger.debug("Set up complete.")

    def test_division_list(self):
        """Test the Compute wrapper with a function that divides each element in a list by a value."""
        logger.debug("test_division_list")

        @compute(
            metadata={
                "description": "Divides each element in a list by a value.",
                "namespace": "climategpt",
                "tags": ["division"],
            },
        )
        def division(x: list, y: list):
            return [i / y for i in x]

        input_list = [6, 12, 18]
        result = division(input_list, 3)

        self.assertEqual(result, Dataset.from_object([2.0, 4.0, 6.0]))

        # Test division by zero (expected ZeroDivisionError)
        with self.assertRaises(ZeroDivisionError):
            division([1, 2, 0], 0)

    def test_square_function(self):
        @compute(
            metadata={
                "description": "Squares a number.",
                "name": "Square Function",
                "namespace": "climategpt",
                "tags": ["math"],
            },
        )
        def square(x):
            return x * x

        result = square(5)
        self.assertEqual(result, 25)

    def test_add_function(self):
        """Test the @compute decorator with a function that adds two numbers."""

        @compute(
            metadata={
                "description": "Adds two numbers.",
                "name": "Add Numbers",
                "namespace": "climategpt",
                "tags": ["math"],
            },
        )
        def add(x, y):
            return x + y

        result = add(3, 4)
        self.assertEqual(result, 7)

    def test_store_override_false(self):
        """Test that a decorator compute with _store=False does not persist the input's blob."""
        purge_blob_store()
        blob_path = Path(get_config_dir()) / "blobs" / NOT_RETAINED_CID

        @compute(metadata={"name": "No Store"}, _store=False)
        def identity(value):
            return value

        self.assertEqual(identity("not retained"), "not retained")
        self.assertFalse(blob_path.exists(), "Input blob should not be stored when _store=False")

    def test_store_override_true(self):
        """Test that a decorator compute with _store=True persists the input's blob."""
        purge_blob_store()
        blob_path = Path(get_config_dir()) / "blobs" / RETAIN_ME_CID

        @compute(metadata={"name": "Force Store"}, _store=True)
        def identity(value):
            return value

        self.assertEqual(identity("retain me"), "retain me")
        self.assertTrue(blob_path.exists(), "Input blob should be stored when _store=True")

    def test_store_override_wins_over_default(self):
        """Test that a decorator's _store=False overrides a True store_all_blobs default."""
        cfg = init(custom_dir=get_config_dir())
        cfg.set_store_all_blobs(True)
        try:
            purge_blob_store()
            blob_path = Path(get_config_dir()) / "blobs" / OVERRIDE_DEFAULT_CID

            @compute(metadata={"name": "Override Default"}, _store=False)
            def identity(value):
                return value

            identity("should not be retained despite the default")
            self.assertFalse(
                blob_path.exists(), "_store=False should override a True store_all_blobs default"
            )
        finally:
            cfg.set_store_all_blobs(False)

    def test_empty_function(self):
        """Test the @compute decorator with an empty function."""

        @compute(
            metadata={
                "description": "Empty function.",
                "namespace": "climategpt",
                "tags": ["empty"],
            },
        )
        def empty_function():
            pass

        with self.assertRaises(UsageError):
            empty_function()

    def test_wrong_argument_type(self):
        """Test the @compute decorator with a function expecting an integer but receiving a string."""

        @compute(
            metadata={
                "description": "Squares a number.",
                "namespace": "climategpt",
                "tags": ["math"],
            },
        )
        def square(x: int):
            return x * x

        with self.assertRaises(TypeError):
            square("five")

    def test_square_list(self):
        """Test the Compute wrapper with a function that squares elements in a list."""

        @compute(
            metadata={
                "description": "Squares elements in a list.",
                "namespace": "climategpt",
                "tags": ["math"],
            },
        )
        def square(x: list):
            return [i * i for i in x]

        input_list = [1, 2, 3]
        result = square(input_list)
        self.assertEqual(result, [1, 4, 9])

    def test_add_list(self):
        """Test the Compute wrapper with a function that adds a value to each element in a list."""

        @compute(
            metadata={
                "description": "Adds a value to each element in a list.",
                "namespace": "climategpt",
                "tags": ["math"],
            },
        )
        def add_value(x: list, value):
            return [i + value for i in x]

        input_list = [1, 2, 3]
        result = add_value(input_list, 5)
        self.assertEqual(result, [6, 7, 8])

    def test_subtract_list(self):
        """Test the Compute wrapper with a function that subtracts a value from each element in a list."""

        @compute(
            metadata={
                "description": "Subtracts a value from each element in a list.",
                "namespace": "climategpt",
                "tags": ["math"],
            },
        )
        def subtract_value(x: list, value):
            return [i - value for i in x]

        input_list = [10, 20, 30]
        result = subtract_value(input_list, 5)
        self.assertEqual(result, [5, 15, 25])

    def test_pow_list(self):
        """Test the Compute wrapper with a function that raises each element in a list to a power."""

        @compute(
            metadata={
                "description": "Raises each element in a list to a power.",
                "namespace": "climategpt",
                "tags": ["math"],
            },
        )
        def power(x: list, y: list):
            return [i**y for i in x]

        input_list = [2, 3, 4]
        result = power(input_list, 2)
        self.assertEqual(result, [4, 9, 16])

    def test_mod_list(self):
        """Test the Compute wrapper with a function that calculates the modulo of each element in a list."""

        @compute(
            metadata={
                "description": "Calculates the modulo of each element in a list.",
                "namespace": "climategpt",
                "tags": ["math"],
            },
        )
        def modulo(x: list, y: list):
            return [i % y for i in x]

        input_list = [10, 13, 17]
        result = modulo(input_list, 3)
        self.assertEqual(result, [1, 1, 2])

    def test_empty_list(self):
        """Test the Compute wrapper with an empty list."""

        @compute(
            metadata={
                "description": "Returns an empty list.",
                "namespace": "climategpt",
                "tags": ["empty"],
            },
        )
        def identity(x: list):
            return x

        result = identity([])
        self.assertEqual(result, [])


if __name__ == "__main__":
    logging.basicConfig(
        level=logging.DEBUG,
        format="(%(asctime)s) %(levelname)s - %(name)s/%(funcName)s: %(message)s",
        handlers=[
            logging.StreamHandler(),
        ],
    )

    logging.debug("Logging configured.")
    unittest.main()
