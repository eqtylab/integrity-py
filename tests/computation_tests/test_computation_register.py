import os
import unittest
from pathlib import Path

from eqty_sdk import CID, Computation
from tests import setup_sdk


class TestComputationRegister(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        os.environ["EQTY_TIMESTAMP"] = "2021-10-01T00:00:00Z"
        os.environ["EQTY_SKIP_PROOF"] = "true"

        setup_sdk()

    def test_register(self):
        computation = (
            Computation.new(name="register", description="custom description")
            .add_input_cid(CID("urn:cid:input1"))
            .add_output_cid(CID("urn:cid:output1"))
            .set_computation_cid(CID("urn:cid:computation"))
        )

        computation.finalize()

        metadata_json_exists = os.path.exists(
            Path("tmp").joinpath(
                "blobs/baga6yaq6ecqtdx2y6lpo5ntsmegrvy44iq6s5ithrijbdvteoy6bzq6ntwzd4"
            )
        )
        self.assertTrue(metadata_json_exists, "Metadata JSON was not created")


if __name__ == "__main__":
    unittest.main()
