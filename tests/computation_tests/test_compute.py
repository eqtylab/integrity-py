import unittest

from eqty_sdk import Compute, Dataset
from tests import setup_sdk


class TestCompute(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_square_function(self):
        """Test the Compute wrapper with a function that squares a number."""

        def square(x):
            return x * x

        compute_square = Compute(square)
        result = compute_square(5)
        self.assertEqual(result, 25)

    def test_add_function(self):
        """Test the Compute wrapper with a function that adds two numbers."""

        def add(x, y):
            return x + y

        compute_add = Compute(add)
        result = compute_add(3, 4)
        self.assertEqual(result, 7)

    def test_empty_function(self):
        """Test the Compute wrapper with an empty function."""

        def empty_function():
            pass

        compute_empty = Compute(empty_function)

        with self.assertRaises(RuntimeError):
            compute_empty()

    def test_wrong_argument_type(self):
        """Test the Compute wrapper with a function expecting an integer but receiving a string."""

        def square(x: int):
            return x * x

        compute_square = Compute(square)

        with self.assertRaises(TypeError):
            compute_square("five")

    def test_division_function(self):
        """Test the Compute wrapper with a function that divides two numbers."""

        def division(x, y):
            return x / y

        compute_division = Compute(division)
        result = compute_division(10, 2)
        self.assertEqual(result, 5)

        # Test division by zero (expected ZeroDivisionError)
        with self.assertRaises(ZeroDivisionError):
            compute_division(10, 0)

    def test_pow_function(self):
        """Test the Compute wrapper with a function that raises a number to a power."""

        def power(x, y):
            return x**y

        compute_power = Compute(power)
        result = compute_power(2, 3)
        self.assertEqual(result, 8)

    def test_mod_function(self):
        """Test the Compute wrapper with a function that calculates the modulo of two numbers."""

        def modulo(x, y):
            return x % y

        compute_modulo = Compute(modulo)
        result = compute_modulo(10, 3)
        self.assertEqual(result, 1)

    def test_dataset_type(self):
        """Test the Compute wrapper with a function expecting Datasets."""

        def mul(x: Dataset, y: Dataset):
            return x * y

        def mulAny(x, y):
            return x * y

        compute_mul = Compute(mul)
        result = compute_mul(Dataset.from_object(5), Dataset.from_object(50))
        self.assertEqual(result, 250)

        compute_mulAny = Compute(mulAny)
        result = compute_mulAny(Dataset.from_object(5), 3)
        self.assertEqual(result, 15)
        result = compute_mulAny(Dataset.from_object([1, 2]), 3)
        self.assertEqual(result, [1, 2, 1, 2, 1, 2])
        result = compute_mulAny(Dataset.from_object([1, 2]), Dataset.from_object(3))
        self.assertEqual(result, [1, 2, 1, 2, 1, 2])


if __name__ == "__main__":
    unittest.main()
