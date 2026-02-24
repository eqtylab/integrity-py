import unittest

from eqty_sdk import Dataset
from tests import setup_sdk


class TestDatasetAsset(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_init_with_valid_value(self):
        """Test initializing Asset with a valid value."""
        data = [1, 2, 3]
        asset = Dataset.from_object(data)
        self.assertEqual(asset._value, data)

    def test_attribute_access_success(self):
        """Test accessing an attribute of the underlying value."""
        data = [1, 2, 3]
        asset = Dataset.from_object(data)
        self.assertEqual(len(asset), 3)

    def test_attribute_access_failure(self):
        """Test accessing a non-existent attribute of the underlying value."""
        data = [1, 2, 3]
        asset = Dataset.from_object(data)
        with self.assertRaises(AttributeError):
            asset.non_existent_attribute  # noqa

    def test_add_with_compatible_types(self):
        """Test adding Asset objects with compatible underlying types (numbers)."""
        data1 = Dataset.from_object(5)
        data2 = Dataset.from_object(3)
        result = data1 + data2
        self.assertEqual(result, 8)

    def test_add_with_incompatible_types(self):
        """Test adding Asset objects with incompatible underlying types (list and string)."""
        data1 = Dataset.from_object([1, 2])
        data2 = Dataset.from_object("string")
        with self.assertRaises(TypeError):
            data1 + data2

    def test_add_with_unsupported_operation(self):
        """Test adding Asset with an object that doesn't support addition."""
        with self.assertRaises(TypeError):
            Dataset.from_object(object())

    def test_other_operations(self):
        """Test other potential operations on the underlying value (assuming multiplication)."""
        data = Dataset.from_object(5)
        result = data * 2
        self.assertEqual(result, 10)

    def test_list_mul_operations(self):
        """Test other potential operations on the underlying value (assuming multiplication)."""
        data = Dataset.from_object([1, 2])
        result = data * 2
        self.assertEqual(result, [1, 2, 1, 2])

    def test_div_operations(self):
        """Test other potential operations on the underlying value (assuming multiplication)."""
        data = Dataset.from_object(6)
        result = data / 2
        self.assertEqual(result, 3)

    def test_division_with_compatible_types(self):
        """Test division of Asset objects with compatible underlying types (numbers)."""
        data = Dataset.from_object(10)
        divisor = 2
        result = data / divisor
        self.assertEqual(result, 5)

    def test_division_with_incompatible_types(self):
        """Test division of Asset objects with incompatible underlying types (list and string)."""
        data1 = Dataset.from_object([1, 2])
        data2 = Dataset.from_object("string")
        with self.assertRaises(TypeError):
            data1 / data2

    def test_division_by_zero(self):
        """Test division of Asset by zero."""
        data = Dataset.from_object(5)
        with self.assertRaises(ZeroDivisionError):
            data / 0

    def test_modulo_with_compatible_types(self):
        """Test modulo of Asset objects with compatible underlying types (integers)."""
        data = Dataset.from_object(10)
        divisor = 3
        result = data % divisor
        self.assertEqual(result, 1)

    def test_modulo_with_datasets(self):
        """Test modulo of Asset objects with compatible Asset objects types (integers)."""
        data = Dataset.from_object(10)
        divisor = Dataset.from_object(3)
        result = data % divisor
        self.assertEqual(result, 1)

    def test_sub_with_compatible_types(self):
        """Test subtraction of Asset objects with compatible underlying types (numbers)."""
        data1 = Dataset.from_object(5)
        data2 = 3
        result = data1 - data2
        self.assertEqual(result, 2)

    def test_sub_with_dataset_types(self):
        """Test subtraction of Asset objects with compatible underlying types (numbers)."""
        data1 = Dataset.from_object(5)
        data2 = Dataset.from_object(3)
        result = data1 - data2
        self.assertEqual(result, 2)


if __name__ == "__main__":
    unittest.main()
