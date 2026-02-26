import json
import os
import sqlite3
import unittest
from typing import List, Optional, Tuple

from eqty_sdk import Asset, Dataset, compute
from tests import get_config_dir, setup_sdk

os.environ["EQTY_SKIP_PROOF"] = "True"
os.environ["EQTY_TIMESTAMP"] = "2021-10-01T00:00:00Z"


@compute()
def return_list(return_none: Optional[bool]) -> Optional[List[int | None]]:
    if return_none:
        return [1, None]
    else:
        return [1, 2]


@compute()
def return_tuple(return_none: bool) -> Tuple[int, Optional[str]]:
    if return_none:
        return (1, None)
    else:
        return (1, "second value")


@compute()
def return_asset_list(return_none: bool) -> List[Optional[Asset]]:
    if return_none:
        return [Dataset.from_object(1), None, Dataset.from_object(2)]
    else:
        return [Dataset.from_object(1), Dataset.from_object(2)]


class TestComputationNoneReturn(unittest.TestCase):
    """Tests that None value inputs and outputs are ignored (no statement is created).
    At least SOMETHING must be returned by the fn. The ComputationStatement schema requires an input and an output cid.
    """

    @classmethod
    def setUpClass(cls):
        setup_sdk()

    def test_default_list(self):
        fn_result = return_list(False)
        self.assertEqual(len(fn_result), 2)

        (input_cids, output_cids) = get_compute_statement_inputs_and_outputs(
            "urn:cid:bagb6qaq6ebjnhlznotpekrnsyu6eecug5myfzccoqby3mxcgquupf4lbylgdi"
        )
        self.assertEqual(len(input_cids), 2)

        self.assertIn(
            "urn:cid:bafkr4igwhpm2qjvpsha75i3rszngjyi64ihrhzdll5jmlgibcntalm5eq4", output_cids
        )
        self.assertIn(
            "urn:cid:bafkr4iebh2nxfekb47zyll5aulin6ptmg6e6ij774sxo6vtkkzn4r4x6hu", output_cids
        )
        self.assertEqual(len(output_cids), 2)

    def test_none_return(self):
        fn_result = return_list(True)
        self.assertEqual(len(fn_result), 2)

        (input_cids, output_cids) = get_compute_statement_inputs_and_outputs(
            "urn:cid:bagb6qaq6ec7qix4hfuv7fsybgih5xlqlxxnyg3mqcaegdcuij4f3ankiierbo"
        )
        self.assertEqual(len(input_cids), 2)

        self.assertIn(
            "urn:cid:bafkr4igwhpm2qjvpsha75i3rszngjyi64ihrhzdll5jmlgibcntalm5eq4", output_cids
        )
        self.assertEqual(len(output_cids), 1)

    def test_none_input(self):
        fn_result = return_list(None)
        self.assertEqual(len(fn_result), 2)

        (input_cids, output_cids) = get_compute_statement_inputs_and_outputs(
            "urn:cid:bagb6qaq6edf5d2tbtmidi7di3zbtk56cw66nbjmpyb7zv3n3hnxjytzjlsmky"
        )
        self.assertIn(
            "urn:cid:bafkr4ie4p254t34l2wgyjqnvo4csgvmtshi2krtfwvqvjppv6ku2b53sty", input_cids
        )
        self.assertEqual(len(input_cids), 1)

        self.assertIn(
            "urn:cid:bafkr4igwhpm2qjvpsha75i3rszngjyi64ihrhzdll5jmlgibcntalm5eq4", output_cids
        )
        self.assertIn(
            "urn:cid:bafkr4iebh2nxfekb47zyll5aulin6ptmg6e6ij774sxo6vtkkzn4r4x6hu", output_cids
        )
        self.assertEqual(len(output_cids), 2)

    def test_tuple_output(self):
        fn_result = return_tuple(False)
        self.assertEqual(len(fn_result), 2)

        (input_cids, output_cids) = get_compute_statement_inputs_and_outputs(
            "urn:cid:bagb6qaq6eci4lr64n3fr4bht6vyagrmjabo4ac4w4r6kfeo4hqqjykvkeia3k"
        )
        self.assertIn(
            "urn:cid:bafkr4igr5bt544kcof6wev4f7fuputvp5tvido3u5rrojm6ajisfmn2iam", input_cids
        )
        self.assertIn(
            "urn:cid:bafkr4ig6prrslbstrhdiviz74apzk7tz3ser7ru3wtoli3zywt2exzah5i", input_cids
        )
        self.assertEqual(len(input_cids), 2)

        self.assertIn(
            "urn:cid:bafkr4igwhpm2qjvpsha75i3rszngjyi64ihrhzdll5jmlgibcntalm5eq4", output_cids
        )
        self.assertIn(
            "urn:cid:bafkr4ifn4xoo3m5gmzudh4vn6add6xkoggbfeson7novq6ibi2iyw3ipqy", output_cids
        )
        self.assertEqual(len(output_cids), 2)

    def test_tuple_with_none(self):
        fn_result = return_tuple(True)
        self.assertEqual(len(fn_result), 2)

        (input_cids, output_cids) = get_compute_statement_inputs_and_outputs(
            "urn:cid:bagb6qaq6edu7tkwkda72rmf6jblr4ll7gufwzhrpaeexc37fbbqatstj43vom"
        )

        self.assertIn(
            "urn:cid:bafkr4igr5bt544kcof6wev4f7fuputvp5tvido3u5rrojm6ajisfmn2iam", input_cids
        )
        self.assertIn(
            "urn:cid:bafkr4ibizgyfduv4ado3slkewyhs4ibkhbsk3nhhrxtf63tmrrdebqqvtm", input_cids
        )
        self.assertEqual(len(input_cids), 2)

        self.assertIn(
            "urn:cid:bafkr4igwhpm2qjvpsha75i3rszngjyi64ihrhzdll5jmlgibcntalm5eq4", output_cids
        )
        self.assertEqual(len(output_cids), 1)

    def test_asset_list(self):
        fn_result = return_asset_list(False)
        self.assertEqual(len(fn_result), 2)

        (input_cids, output_cids) = get_compute_statement_inputs_and_outputs(
            "urn:cid:bagb6qaq6ebwzsqumshbmtsuwh4pclrhpntobqalhelrhl5wyzhyin2ubrolau"
        )
        self.assertEqual(len(input_cids), 2)
        self.assertEqual(len(output_cids), 2)

        self.assertIn(
            "urn:cid:bafkr4igwhpm2qjvpsha75i3rszngjyi64ihrhzdll5jmlgibcntalm5eq4", output_cids
        )
        self.assertIn(
            "urn:cid:bafkr4iebh2nxfekb47zyll5aulin6ptmg6e6ij774sxo6vtkkzn4r4x6hu", output_cids
        )

    def test_asset_none_list(self):
        fn_result = return_asset_list(True)
        self.assertEqual(len(fn_result), 3)

        (input_cids, output_cids) = get_compute_statement_inputs_and_outputs(
            "urn:cid:bagb6qaq6eas73lbi3hosafpu7p3i77fdlwnfqybazxxneiipy37mydl5azruo"
        )
        self.assertEqual(len(input_cids), 2)
        self.assertEqual(len(output_cids), 2)

        self.assertIn(
            "urn:cid:bafkr4igwhpm2qjvpsha75i3rszngjyi64ihrhzdll5jmlgibcntalm5eq4", output_cids
        )
        self.assertIn(
            "urn:cid:bafkr4iebh2nxfekb47zyll5aulin6ptmg6e6ij774sxo6vtkkzn4r4x6hu", output_cids
        )


if __name__ == "__main__":
    unittest.main()


def get_compute_statement_inputs_and_outputs(id: str) -> Tuple[List[str], List[str]]:
    db_file = get_config_dir().joinpath("graphs.db")

    with sqlite3.connect(db_file) as conn:
        cursor = conn.cursor()
        cursor.execute("SELECT statement FROM computation_statements WHERE id = ?", (id,))
        result = cursor.fetchone()

        if not result:
            print("NO STATEMENT FOUND")
            return ([], [])

        statement = json.loads(result[0])
        inputs = statement.get("input", [])
        if isinstance(inputs, str):
            inputs = [inputs]

        outputs = statement.get("output", [])
        if isinstance(outputs, str):
            outputs = [outputs]

    return (inputs, outputs)
