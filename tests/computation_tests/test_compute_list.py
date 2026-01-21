import unittest

from eqty_sdk import Compute
from tests import setup_sdk


class TestComputeList(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_square_list(self):
        """Test the Compute wrapper with a function that squares elements in a list."""

        def square(x: list):
            return [i * i for i in x]

        compute_square = Compute(square)
        input_list = [1, 2, 3]
        result = compute_square(input_list)
        self.assertEqual(result, [1, 4, 9])

    def test_add_list(self):
        """Test the Compute wrapper with a function that adds a value to each element in a list."""

        def add_value(x: list, value):
            return [i + value for i in x]

        compute_add = Compute(add_value)
        input_list = [1, 2, 3]
        result = compute_add(input_list, 5)
        self.assertEqual(result, [6, 7, 8])

    def test_subtract_list(self):
        """Test the Compute wrapper with a function that subtracts a value from each element in a list."""

        def subtract_value(x: list, value):
            return [i - value for i in x]

        compute_subtract = Compute(subtract_value)
        input_list = [10, 20, 30]
        result = compute_subtract(input_list, 5)  # Pass the value as a separate argument
        self.assertEqual(result, [5, 15, 25])

    def test_division_list(self):
        """Test the Compute wrapper with a function that divides each element in a list by a value."""

        def division(x: list, y: list):
            return [i / y for i in x]

        compute_division = Compute(division)
        input_list = [6, 12, 18]
        result = compute_division(input_list, 3)
        self.assertEqual(result, [2.0, 4.0, 6.0])

        # Test division by zero (expected ZeroDivisionError)
        with self.assertRaises(ZeroDivisionError):
            compute_division([1, 2, 0], 0)

    def test_pow_list(self):
        """Test the Compute wrapper with a function that raises each element in a list to a power."""

        def power(x: list, y: list):
            return [i**y for i in x]

        compute_power = Compute(power)
        input_list = [2, 3, 4]
        result = compute_power(input_list, 2)
        self.assertEqual(result, [4, 9, 16])

    def test_mod_list(self):
        """Test the Compute wrapper with a function that calculates the modulo of each element in a list."""

        def modulo(x: list, y: list):
            return [i % y for i in x]

        compute_modulo = Compute(modulo)
        input_list = [10, 13, 17]
        result = compute_modulo(input_list, 3)
        self.assertEqual(result, [1, 1, 2])

    def test_empty_list(self):
        """Test the Compute wrapper with an empty list."""

        def identity(x: list):
            return x

        compute_identity = Compute(identity)
        result = compute_identity([])
        self.assertEqual(result, [])

    def test_wrong_argument_type(self):
        """Test the Compute wrapper with a function expecting a number but receiving a string in the list."""

        def square(x: int):
            return x * x

        compute_square = Compute(square)

        with self.assertRaises(TypeError):
            compute_square(["one", 2, 3])


if __name__ == "__main__":
    unittest.main()
