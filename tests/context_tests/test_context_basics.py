import unittest

from eqty_sdk import Computation, Compute, Context, Dataset
from tests import setup_sdk


class TestContextBasics(unittest.TestCase):
    """Checks that the context interface is useable."""

    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_dataset(self):
        """Test initializing Dataset asset with context."""
        ctx = Context.new("test_dataset")

        # verigy original context
        self.assertFalse(hasattr(ctx, "tests_dataset"))
        self.assertFalse(hasattr(ctx, "parent_graph"))

        dataset_cid = "bafkr4iff6c2wj5hsaff57e4svjqwziq75vkg72z6zdp5stakweld7lt7ge"

        asset = Dataset.with_context(ctx).from_cid(dataset_cid)

        # Verify the asset was created
        self.assertIsNotNone(asset)
        self.assertIsNotNone(asset.cid)
        self.assertTrue(len(asset.statement_ids) > 0)

        # Check there are statements with the expected __project_id attribute
        (project_statements, _) = statements.retrieve_statements(
            f"attributes.__project_id == '{project_id}'"
        )
        self.assertTrue(
            len(project_statements) >= 3,
            "Expected at least 3 statements (Data, Metadata, Assoc + 3 VCs if VCs are enabled) with the matching project_id attribute",
        )

        # Check that association statements were created when project_id is present
        # The asset should have created an association statement linking the asset CID to the project UUID
        expected_subject = f"urn:cid:{asset.cid}"
        expected_association = f"urn:uuid:{project_id}"

        # Query for all association statements
        filter_query = "statementType == 'AssociationRegistration'"
        (retrieved_statements, _) = statements.retrieve_statements(filter_query)

        # Check the retrieved association statements to find one with our subject and association
        matching_statement_found = False
        matching_statement_in_asset_ids = False

        for stmt in retrieved_statements:
            stmt_id = stmt.get("@id")
            stmt_subject = stmt.get("subject")
            stmt_association = stmt.get("association")

            # Check if this statement matches our expected subject and association
            if stmt_subject == expected_subject and stmt_association == expected_association:
                matching_statement_found = True
                # Also check if this statement ID is in the asset's statement_ids that it's keeping track of
                if stmt_id and stmt_id in asset.statement_ids:
                    matching_statement_in_asset_ids = True
                break

        self.assertTrue(
            matching_statement_found,
            f"Expected to find association statement linking {expected_subject} to {expected_association}",
        )

        self.assertTrue(
            matching_statement_in_asset_ids,
            "Association statement should be included in asset's statement_ids",
        )

    def test_computation(self):
        """Test initializing Computation with context."""
        ctx = Context.new("test_computation")

        computation = (
            Computation.with_context(ctx)
            .new(
                name="Test Computation",
                description="A test computation created in TestContextBasics",
            )
            .add_input_cid("bafkreigh2akiscaildc3n6cnq3g5u5j4f6l5j5x4ux7z4x3t3j3t5v7szy")
            .add_output_cid("bafkreigh2akiscaildc3n6cnq3g5u5j4f6l5j5x4ux7z4x3t3j3t5v7szy")
            .finalize()
        )

        # Verify the computation was created
        self.assertIsNotNone(computation)
        self.assertIsNotNone(computation._metadata)
        self.assertEqual(computation._metadata.name, "Test Computation")
        self.assertEqual(len(computation._input_cids), 1)
        self.assertEqual(len(computation._output_cids), 1)

        # Check there are statements with the expected __project_id attribute
        (project_statements, _) = statements.retrieve_statements(
            f"attributes.__project_id == '{project_id}'"
        )
        self.assertTrue(
            len(project_statements) >= 2,
            "Expected at least 2 statements (Computation, Metadata + 2 VCs if VCs are enabled) with the matching project_id attribute",
        )

    def test_compute(self):
        """Test initializing Compute with context."""

        def add(x, y):
            return x + y

        ctx = Context.new("test_compute")

        wrapped_add = Compute(add, ctx=ctx)

        result = wrapped_add(5, 10)
        self.assertEqual(result, 15)

        # Check there are statements with the expected __project_id attribute
        (project_statements, _) = statements.retrieve_statements(
            f"attributes.__project_id == '{project_id}'"
        )
        self.assertTrue(
            len(project_statements) >= 2,
            "Expected at least 2 statements (Computation, Metadata + 2 VCs if VCs are enabled) with the matching project_id attribute",
        )

    def test_compute_decorator(self):
        """Test using the @compute decorator with context."""
        from eqty_sdk import compute

        ctx = Context.new("test_compute_decorator")

        @compute(
            metadata={
                "description": "Adds two numbers.",
                "name": "Add Numbers",
            },
            ctx=ctx,
        )
        def add(x, y):
            return x + y

        result = add(3, 4)
        self.assertEqual(result, 7)

        # Check there are statements with the expected __project_id attribute
        (project_statements, _) = statements.retrieve_statements(
            f"attributes.__project_id == '{project_id}'"
        )
        self.assertTrue(
            len(project_statements) >= 2,
            "Expected at least 2 statements (Computation, Metadata + 2 VCs if VCs are enabled) with the matching project_id attribute",
        )


if __name__ == "__main__":
    unittest.main()
