import json
import tempfile
import unittest
import uuid
from datetime import datetime, timezone
from pathlib import Path

from eqty_sdk import CID, Computation, Context, verify_statement
from eqty_sdk.statements import otel_span_urn
from tests import setup_sdk

COMPUTATION_CONTEXT = "urn:cid:bafkr4iduavib2lgjcmx5n4ydefbbc62ngyannzi566zv5fsozgbwwfw27u"
OTEL_SPAN_NAMESPACE = uuid.UUID("3eeccd21-6917-4419-a92d-893e3fd3a821")
TRACE = "01a1401d684b7f40a85aba2fa15ba51d"
ROOT = "a85aba2fa15ba51d"
CHILD = "93460e50e31602c0"
INPUT = "urn:cid:bafkr4ia3wmrvedxwkjm6jfmtqy2bdcpi47hv5bni7twshohepck3gsgodi"
OUTPUT = "urn:cid:bafkr4ifqnz4knhvvgorsjl6hwhtmzfkfurebfslr3mwe2bqygcgu4bq3wi"


class TestComputationSpan(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_span_urn_is_a_v5_uuid_of_trace_and_span(self):
        expected = uuid.uuid5(OTEL_SPAN_NAMESPACE, f"{TRACE}/{CHILD}")
        self.assertEqual(otel_span_urn(TRACE, CHILD), f"urn:uuid:{expected}")
        self.assertEqual(otel_span_urn(TRACE.upper(), CHILD.upper()), f"urn:uuid:{expected}")
        with self.assertRaises(ValueError):
            otel_span_urn("01a1401d", CHILD)

    def test_child_names_its_parent_and_exports_verifiable(self):
        ctx = Context.new("computation-span")
        factory = Computation.with_context(ctx)

        # The child finalizes first and names a parent that has not been registered yet
        child = (
            factory.new(name="execute_tool Bash")
            .add_input_cid(CID(INPUT))
            .add_output_cid(CID(OUTPUT))
            .set_start_time(datetime(2026, 10, 15, 15, 10, 35, 645000, tzinfo=timezone.utc))
            .set_end_time("2026-10-15T15:10:35.758Z")
            .set_span(TRACE, CHILD, ROOT)
            .finalize()
        )
        parent = (
            factory.new(name="invoke_agent")
            .add_input_cid(CID(INPUT))
            .add_output_cid(CID(OUTPUT))
            .set_start_time("2026-10-15T15:10:31.479Z")
            .set_end_time("2026-10-15T15:10:46.247Z")
            .set_span(TRACE, ROOT)
            .finalize()
        )

        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / "manifest.json"
            ctx.export(dest)
            manifest = json.loads(dest.read_text())

        statements = manifest["statements"]
        child_statement = statements[str(child.statement_id)]
        parent_statement = statements[str(parent.statement_id)]

        self.assertEqual(child_statement["@context"], COMPUTATION_CONTEXT)
        self.assertEqual(child_statement["startTime"], "2026-10-15T15:10:35.645000+00:00")
        self.assertEqual(child_statement["endTime"], "2026-10-15T15:10:35.758Z")
        self.assertEqual(child_statement["span"], otel_span_urn(TRACE, CHILD))
        self.assertEqual(child_statement["parentSpan"], parent_statement["span"])
        self.assertNotIn("parentSpan", parent_statement)
        self.assertNotIn("subcomputations", parent_statement)

        self.assertIn(COMPUTATION_CONTEXT, manifest["contexts"])
        for statement in (child_statement, parent_statement):
            self.assertTrue(verify_statement(json.dumps(statement), manifest["contexts"]))

    def test_plain_computation_keeps_common_context(self):
        computation = (
            Computation.new(name="plain")
            .add_input_cid(CID("urn:cid:input1"))
            .add_output_cid(CID("urn:cid:output1"))
            .finalize()
        )
        self.assertIsNotNone(computation.statement_id)

    def test_invalid_span_ids_are_rejected(self):
        with self.assertRaises(ValueError):
            Computation.new(name="bad").set_span(TRACE, "not-a-span-id")

    def test_naive_datetime_is_rejected(self):
        with self.assertRaises(ValueError):
            Computation.new(name="naive").set_start_time(datetime(2026, 10, 15, 15, 10, 35))


if __name__ == "__main__":
    unittest.main()
