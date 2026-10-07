"""Checks for the rewrites in render_api_docs.py.

    .venv-docs/bin/python -m unittest scripts/test_render_api_docs.py

Needs the same griffe and griffe2md as the render script. Not collected by `just test-py`,
which discovers under tests/ only.
"""

import re
import subprocess
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import render_api_docs as r  # noqa: E402


class CodeSpanCrossReferences(unittest.TestCase):
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


class Blocks(unittest.TestCase):
    PAGE = (
        "Intro.\n\n"
        "{/* generated api eqty_sdk.init */}\n"
        "stale\n"
        "{/* end generated */}\n\n"
        "{/* generated file examples/a.py python */}\n"
        "{/* end generated */}\n"
    )

    def test_api_and_file_blocks_are_filled_and_markers_kept(self) -> None:
        out, used = r.fill(self.PAGE, {"eqty_sdk.init": "### `init`\n"}, lambda p: "x = 1\n")
        self.assertEqual(
            out,
            "Intro.\n\n"
            "{/* generated api eqty_sdk.init */}\n"
            "### `init`\n"
            "{/* end generated */}\n\n"
            "{/* generated file examples/a.py python */}\n"
            "```python\nx = 1\n```\n"
            "{/* end generated */}\n",
        )
        self.assertEqual(used, {"eqty_sdk.init"})

    def test_filling_twice_changes_nothing(self) -> None:
        once, _ = r.fill(self.PAGE, {"eqty_sdk.init": "A\n"}, lambda p: "x\n")
        twice, _ = r.fill(once, {"eqty_sdk.init": "A\n"}, lambda p: "x\n")
        self.assertEqual(once, twice)

    def test_file_block_without_lang_is_inserted_as_markdown(self) -> None:
        page = "{/* generated file docs/generated/list.md */}\n{/* end generated */}\n"
        out, _ = r.fill(page, {}, lambda p: "- one\n")
        self.assertIn("\n- one\n{/* end", out)

    def test_unknown_target_fails(self) -> None:
        page = "{/* generated api eqty_sdk.nope */}\n{/* end generated */}\n"
        with self.assertRaises(SystemExit) as ctx:
            r.fill(page, {}, lambda p: "")
        msg = str(ctx.exception)
        self.assertIn("eqty_sdk.nope", msg)
        self.assertIn("not in scripts/api-directives.json. Add it there", msg)

    def test_unclosed_marker_fails(self) -> None:
        with self.assertRaises(SystemExit) as ctx:
            r.fill("{/* generated api eqty_sdk.init */}\nno end\n", {"eqty_sdk.init": ""}, str)
        self.assertIn("needs a {/* end generated */} line after it", str(ctx.exception))

    def test_stray_end_marker_fails(self) -> None:
        with self.assertRaises(SystemExit):
            r.fill("prose\n{/* end generated */}\n", {}, str)

    def test_missing_repo_file_fails_naming_it(self) -> None:
        with self.assertRaises(SystemExit) as ctx:
            r.read_repo_file("examples/does-not-exist.py")
        msg = str(ctx.exception)
        self.assertIn("examples/does-not-exist.py", msg)
        self.assertIn("If it was renamed or moved, update the generated file marker", msg)


class CheckMode(unittest.TestCase):
    """main() against a temporary content folder and a one-entry fake API."""

    def setUp(self) -> None:
        import tempfile
        from unittest import mock

        self.dir = Path(tempfile.mkdtemp())
        self.page = self.dir / "p.mdx"
        self.page.write_text("{/* generated api eqty_sdk.init */}\n{/* end generated */}\n")
        for name, value in {
            "CONTENT": self.dir,
            "load_package": lambda: None,
            "load_table": lambda: [{"target": "eqty_sdk.init"}],
            "render": lambda pkg, target, options: "### `eqty_sdk.init`\n",
        }.items():
            patcher = mock.patch.object(r, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)

    def test_check_fails_on_a_stale_page_and_leaves_it_alone(self) -> None:
        before = self.page.read_text()
        self.assertEqual(r.main(["--check"]), 1)
        self.assertEqual(self.page.read_text(), before)

    def test_check_passes_after_a_fill(self) -> None:
        self.assertEqual(r.main([]), 0)
        self.assertEqual(r.main(["--check"]), 0)

    def test_a_target_missing_from_the_code_says_how_to_fix(self) -> None:
        from unittest import mock

        def gone(pkg, target, options):
            raise SystemExit(f"render_api_docs: {target} is not in eqty_sdk")

        with mock.patch.object(r, "render", gone):
            with self.assertRaises(SystemExit) as ctx:
                r.main([])
        msg = str(ctx.exception)
        self.assertIn("eqty_sdk.init is not in eqty_sdk. If it was renamed or removed", msg)
        self.assertIn("scripts/api-directives.json", msg)
        self.assertIn("{/* generated api ... */} marker", msg)

    def test_an_unused_directive_says_how_to_fix(self) -> None:
        from unittest import mock

        table = [{"target": "eqty_sdk.init"}, {"target": "eqty_sdk.extra"}]
        with mock.patch.object(r, "load_table", lambda: table):
            with self.assertRaises(SystemExit) as ctx:
                r.main([])
        msg = str(ctx.exception)
        self.assertIn("no page uses eqty_sdk.extra", msg)
        self.assertIn("remove the entry from scripts/api-directives.json", msg)

    def test_a_broken_page_is_named(self) -> None:
        (self.dir / "broken.mdx").write_text("prose\n{/* end generated */}\n")
        with self.assertRaises(SystemExit) as ctx:
            r.main([])
        self.assertIn("broken.mdx", str(ctx.exception))


class MissingRenderer(unittest.TestCase):
    def test_a_missing_griffe_says_to_run_just_install(self) -> None:
        script = Path(__file__).resolve().parent / "render_api_docs.py"
        code = (
            "import runpy, sys; sys.modules['griffe'] = None; "
            f"runpy.run_path({str(script)!r}, run_name='__main__')"
        )
        out = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True)
        self.assertEqual(out.returncode, 1)
        self.assertIn("griffe is not installed. Run `just install`", out.stderr)


class Fences(unittest.TestCase):
    def test_fence_outgrows_backticks_in_the_code(self) -> None:
        out = r.fence('s = """\n```\n"""\n', "python")
        self.assertTrue(out.startswith("````python\n"))
        self.assertTrue(out.endswith("\n````\n"))


class CodeTags(unittest.TestCase):
    def test_code_tags_become_backtick_spans(self) -> None:
        out = r.mdx_safe(r"<code>[Optional](#typing.Optional)\[[UUID](#uuid.UUID)\]</code>", {})
        self.assertEqual(out, "`Optional[UUID]`")


if __name__ == "__main__":
    unittest.main()
