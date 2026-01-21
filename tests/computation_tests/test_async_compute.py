import asyncio
import unittest

from eqty_sdk import Compute
from tests import setup_sdk


class AsyncTestCase(unittest.IsolatedAsyncioTestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()


class TestAsyncCompute(AsyncTestCase):
    async def test_async_square_function(self):
        """Test the with an async function that squares a number."""

        async def square(x):
            await asyncio.sleep(0.2)
            return x * x

        compute_square = Compute(square)
        result = await compute_square(5)
        self.assertEqual(result, 25)

    async def test_number_generator(self):
        """Test with a function that generates numbers async."""

        async def async_gen():
            for i in range(3):
                yield i

        compute = Compute(async_gen)

        results = []
        async for value in compute():
            results.append(value)

        self.assertEqual(results, [0, 1, 2])

    async def test_str_generator(self):
        """Test with a function that generates strings async."""

        async def async_gen():
            results = ["one", "two", "three"]
            for r in results:
                yield r

        compute = Compute(async_gen)

        results = []
        async for value in compute():
            results.append(value)

        self.assertEqual(results, ["one", "two", "three"])

    async def test_obj_generator(self):
        """Test with a function that generates objects async."""

        class ResultObj:
            value: int
            name: str

            def __init__(self, value: int, name: str):
                self.value = value
                self.name = name

        async def async_gen():
            names = ["one", "two", "three"]
            values = [1, 2, 3]
            for o in range(len(names)):
                yield ResultObj(values[o], names[o])

        compute = Compute(async_gen)

        results = []
        async for value in compute():
            self.assertIsInstance(value, ResultObj)
            results.append(value)

        for i in range(3):
            self.assertEqual(results[i].value, i + 1)
            self.assertEqual(results[i].name, ["one", "two", "three"][i])


if __name__ == "__main__":
    unittest.main()
