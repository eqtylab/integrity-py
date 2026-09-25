"""Checks for the rewrites in render_api_docs.py.

    .venv-docs/bin/python -m unittest scripts/test_render_api_docs.py

Needs the same griffe and griffe2md as the render script. Not collected by `just test-py`,
which discovers under tests/ only.
"""

import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import render_api_docs as r  # noqa: E402


class CodeSpanCrossReferences(unittest.TestCase):
    def test_every_link_inside_a_code_span_is_unwrapped(self) -> None:
        out = r.mdx_safe(r"<code>[Optional](#typing.Optional)\[[UUID](#uuid.UUID)\]</code>", {})
        self.assertEqual(out, r"<code>Optional\[UUID\]</code>")

    def test_links_to_headings_in_the_partials_are_rewritten(self) -> None:
        out = r.mdx_safe(
            "[**load**](#eqty_sdk._rust.Signer.load)", {"eqty_sdk._rust.Signer.load": "x"}
        )
        self.assertEqual(out, "[**load**](#x)")


class Signatures(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.pkg = r.load_package()
        cls.options = {d["target"]: d.get("options", {}) for d in r.load_table()}

    def render(self, target: str) -> str:
        return r.render(self.pkg, target, self.options[target])

    def test_unannotated_kwargs_do_not_inherit_the_previous_type(self) -> None:
        cases = {
            "eqty_sdk.asset.asset.TypedAsset.from_object": "from_object(obj: Any, _store: Optional[bool] = None, **kwargs) -> TypedAssetT",
            "eqty_sdk.asset.asset.TypedAsset.from_cid": "from_cid(cid: CID, **kwargs) -> TypedAssetT",
            "eqty_sdk.statements.Association": "new(subject: CID | DID | UUID, association_type: PyAssociationType, **kwargs) -> Association",
        }
        for target, expected in cases.items():
            with self.subTest(target=target):
                md = self.render(target)
                self.assertIn(expected, md)
                self.assertNotRegex(md, r"\*\*kwargs: ")

    def test_class_signatures_do_not_end_in_none(self) -> None:
        md = self.render("eqty_sdk.compute")
        self.assertNotRegex(md, re.compile(r"^Compute\(.*\) -> None$", re.M))
        self.assertRegex(
            md, re.compile(r"^Compute\(func: Callable\[\.\.\., Any\], .*\*\*kwargs\)$", re.M)
        )

    def test_a_function_that_returns_none_keeps_it(self) -> None:
        self.assertIn("purge_blob_store() -> None", self.render("eqty_sdk.purge_blob_store"))

    def test_signatures_stay_on_one_line_when_black_is_installed(self) -> None:
        md = self.render("eqty_sdk._rust.statements.add_association_statement")
        fence = re.search(r"```python\n(.*?)\n```", md, re.S)
        assert fence is not None
        self.assertNotIn("\n", fence.group(1))


if __name__ == "__main__":
    unittest.main()
