import itertools
import os
import unittest
from datetime import datetime, timezone
from pathlib import Path
from unittest import mock

from eqty_sdk import Compute, compute
from tests import get_statement_count_by_type, setup_sdk


def dummy_func(count: int) -> str:
    return f"output {count}"


def set_attributes():
    return {"unit test": "ComputeIssueVC"}


@compute(attributes=set_attributes)
def decorator_default(count: int) -> str:
    return f"output {count}"


@compute(_skip_proof=True)
def decorator_no_vc(count: int) -> str:
    return f"output {count}"


@compute(_skip_proof=False)
def decorator_forced_vc(count: int) -> str:
    return f"output {count}"


tmp_dir = Path("tmp")

# Registering identical content again with the same timestamp reuses the existing
# credential instead of issuing a new one. To count every credential, each test gets its own
# timestamp and each call its own argument, so no statement repeats. The statement store
# outlives a test run, so the timestamp must also differ from earlier runs.
_args = itertools.count()


class ComputeIssueVC(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def setUp(self):
        env = mock.patch.dict(
            os.environ,
            {"EQTY_TIMESTAMP": datetime.now(timezone.utc).isoformat(timespec="microseconds")},
        )
        env.start()
        self.addCleanup(env.stop)
        self._baseline_vc_count = get_statement_count_by_type("CredentialRegistration") or 0

    def _vc_delta(self) -> int:
        current = get_statement_count_by_type("CredentialRegistration") or 0
        return current - self._baseline_vc_count

    def test_compute_default(self):
        """Check that by default a proof is created."""
        os.environ.pop("EQTY_SKIP_PROOF", None)
        compute = Compute(dummy_func)
        compute(next(_args))
        self.assertFalse(compute._skip_proof, "Compute default skip_proof failed")

        decorator_default(next(_args))
        self.assertEqual(self._vc_delta(), 16, "Number of new VCs is incorrect")

    def test_compute_env_var_enable(self):
        """Check that the env var doesn't default on a bad setting."""
        os.environ["EQTY_SKIP_PROOF"] = "not_true"
        compute = Compute(dummy_func)
        compute(next(_args))
        self.assertFalse(compute._skip_proof, "Compute env var enable skip_proof failed")

        decorator_default(next(_args))
        self.assertEqual(self._vc_delta(), 16, "Number of new VCs is incorrect")

    def test_compute_override(self):
        """Check that passing setting _skip_proof param doesn't create vcs."""
        compute = Compute(dummy_func, _skip_proof=True)
        compute(next(_args))
        self.assertTrue(compute._skip_proof, "Compute override skip_proof failed")

        decorator_no_vc(next(_args))
        self.assertEqual(self._vc_delta(), 0, "There should not be any new VCs")

    def test_compute_env_var_with_forced_vc(self):
        """Check that the parameter overrides the env var."""
        os.environ["EQTY_SKIP_PROOF"] = "True"

        decorator_default(next(_args))
        decorator_forced_vc(next(_args))
        self.assertEqual(self._vc_delta(), 8, "The explicit VCs were not created")


if __name__ == "__main__":
    unittest.main()
