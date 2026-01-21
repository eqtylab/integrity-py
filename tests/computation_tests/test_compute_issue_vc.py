import os
import unittest
from pathlib import Path

from eqty_sdk import Compute, compute
from tests import get_statement_count_by_type, reset_statement_db, setup_sdk


def dummy_func(count: int) -> str:
    return count.__str__()


def set_attributes():
    return {"unit test": "ComputeIssueVC"}


@compute(attributes=set_attributes)
def decorator_default(count: int) -> str:
    return count.__str__()


@compute(skip_proof=True)
def decorator_no_vc(count: int) -> str:
    return count.__str__()


@compute(skip_proof=False)
def decorator_forced_vc(count: int) -> str:
    return count.__str__()


tmp_dir = Path("tmp")


class ComputeIssueVC(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def setUp(self):
        reset_statement_db()
        pass

    def test_compute_default(self):
        """Check that by default a proof is created."""
        os.environ.pop("EQTY_SKIP_PROOF", None)
        compute = Compute(dummy_func)
        compute(5)
        self.assertFalse(compute._skip_proof, "Compute default skip_proof failed")

        decorator_default(5)
        vc_count = get_statement_count_by_type("CredentialRegistration")
        self.assertEqual(vc_count, 16, "Number of VCs is incorrect")

    def test_compute_env_var_enable(self):
        """Check that the env var doesn't default on a bad setting."""
        os.environ["EQTY_SKIP_PROOF"] = "not_true"
        compute = Compute(dummy_func)
        compute(5)
        self.assertFalse(compute._skip_proof, "Compute env var enable skip_proof failed")

        decorator_default(5)
        vc_count = get_statement_count_by_type("CredentialRegistration")
        self.assertEqual(vc_count, 16, "Number of VCs is incorrect")

    def test_compute_override(self):
        """Check that passing setting skip_proof param doesn't create vcs."""
        compute = Compute(dummy_func, skip_proof=True)
        compute(5)
        self.assertTrue(compute._skip_proof, "Compute override skip_proof failed")

        decorator_no_vc(5)
        vc_count = get_statement_count_by_type("CredentialRegistration")
        self.assertEqual(vc_count, 0, "There should not be any VCs")

    def test_compute_env_var(self):
        """Check the case sensitivity of the env var."""
        os.environ["EQTY_SKIP_PROOF"] = "TrUE"
        compute = Compute(dummy_func)
        compute(5)
        self.assertTrue(compute._skip_proof, "Compute env var disable skip_proof failed")

        decorator_default(5)
        vc_count = get_statement_count_by_type("CredentialRegistration")
        self.assertEqual(vc_count, 0, "There should not be any VCs")

    def test_compute_env_var_with_forced_vc(self):
        """Check that the parameter overrides the env var."""
        os.environ["EQTY_SKIP_PROOF"] = "True"

        decorator_default(5)
        decorator_forced_vc(5)
        vc_count = get_statement_count_by_type("CredentialRegistration")
        self.assertEqual(vc_count, 8, "The explicit VCs were not created")


if __name__ == "__main__":
    unittest.main()
