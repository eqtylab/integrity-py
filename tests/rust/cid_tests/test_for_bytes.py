import shutil
import tempfile
import unittest

import eqty_sdk._rust as core


class CidBytes(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp_dir = tempfile.mkdtemp(prefix=f"{cls.__name__}_")
        core.context.init(cls.temp_dir)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.temp_dir)

    def test_file_bytes(self):
        b = b"Hello World"
        cid = core.cid.compute_cid_for_bytes(b)
        self.assertEqual(
            cid, "bafkr4icb7a4uceploe5cefs4i3eqvohq7wjztsjafd6w2kejiszd75n7oy", "Bytes CID mismatch"
        )


if __name__ == "__main__":
    unittest.main()
