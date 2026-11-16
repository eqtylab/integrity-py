import json
import os
import tempfile
import unittest
from pathlib import Path

from eqty_sdk import CID, Code, Computation, Context, Dataset, Model
from tests import get_statement_count_by_type, setup_sdk

MODEL_CID = CID("bafkr4icqw77khu73vgw74jpnlnep37ec3l6jd4lg5kvw2letvqjhgk6jmi")


class RepeatedRequestTests(unittest.TestCase):
    """Each request gets its own context, registers some of the same assets, then exports
    and deletes the context. Statement IDs hash a timestamp rounded to the second, so
    requests within the same second register identical statements for the repeated assets.
    The fixed EQTY_TIMESTAMP makes every request land in the same second.
    """

    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def setUp(self):
        self._prev_timestamp = os.environ.get("EQTY_TIMESTAMP")
        os.environ["EQTY_TIMESTAMP"] = "2026-11-16T12:00:00Z"

    def tearDown(self):
        if self._prev_timestamp is None:
            os.environ.pop("EQTY_TIMESTAMP", None)
        else:
            os.environ["EQTY_TIMESTAMP"] = self._prev_timestamp

    def _request(self, i: int, out_dir: Path) -> dict:
        ctx = Context.new(f"repeated-request-{i}")
        code = Code.with_context(ctx).from_object("def handler(): ...", name="handler")
        model = Model.with_context(ctx).from_cid(MODEL_CID, name="model")
        inp = Dataset.with_context(ctx).from_object(f"repeated request input {i}", name="input")
        out = Dataset.with_context(ctx).from_object(f"repeated request output {i}", name="output")
        Computation.with_context(ctx).new(name="call").add_input_cid(
            [code.cid, model.cid, inp.cid]
        ).add_output_cid(out.cid).finalize()

        path = out_dir / f"manifest-{i}.json"
        ctx.export(path)
        ctx.delete_tree()
        return json.loads(path.read_text())

    def test_manifests_do_not_include_credentials_from_deleted_requests(self):
        credentials_before = get_statement_count_by_type("CredentialRegistration") or 0
        with tempfile.TemporaryDirectory() as tmp:
            manifests = [self._request(i, Path(tmp)) for i in range(4)]

        sizes = [len(m["statements"]) for m in manifests]
        self.assertEqual(sizes, [sizes[0]] * len(sizes), "manifest size grew across requests")

        for manifest in manifests:
            subjects = [
                s["credential"]["credentialSubject"]["id"]
                for s in manifest["statements"].values()
                if s["@type"] == "CredentialRegistration"
            ]
            self.assertEqual(len(subjects), len(set(subjects)), "a statement has two credentials")

        credentials_after = get_statement_count_by_type("CredentialRegistration") or 0
        self.assertEqual(
            credentials_after, credentials_before, "delete_tree left credentials behind"
        )


if __name__ == "__main__":
    unittest.main()
