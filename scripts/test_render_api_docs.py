"""Checks for the rewrites in render_api_docs.py.

    poetry run python -m unittest scripts/test_render_api_docs.py

Needs the same griffe and griffe2md as the render script, which `just install` provides. Not
collected by `just test-py`, which discovers under tests/ only.
"""

import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

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
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.dir = Path(tmp.name)
        self.page = self.dir / "p.mdx"
        self.page.write_text("{/* generated api eqty_sdk.init */}\n{/* end generated */}\n")
        for name, value in {
            "CONTENT": self.dir,
            "load_package": lambda: None,
            "load_table": lambda: [{"target": "eqty_sdk.init"}],
            "render": lambda pkg, target, options, **kw: "### `eqty_sdk.init`\n",
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

    def test_main_asks_render_for_the_docs_advice(self) -> None:
        seen = []

        def spy(pkg, target, options, **kw):
            seen.append(kw)
            return "### `eqty_sdk.init`\n"

        with mock.patch.object(r, "render", spy):
            r.main([])
        self.assertEqual(seen, [{"advise": True}])

    def test_render_errors_pass_through_unchanged(self) -> None:
        def unresolved(pkg, target, options, **kw):
            raise SystemExit("render_api_docs: own advice")

        with mock.patch.object(r, "render", unresolved):
            with self.assertRaises(SystemExit) as ctx:
                r.main([])
        self.assertEqual(str(ctx.exception), "render_api_docs: own advice")

    def test_an_unused_directive_says_how_to_fix_and_writes_nothing(self) -> None:
        before = self.page.read_text()
        table = [{"target": "eqty_sdk.init"}, {"target": "eqty_sdk.extra"}]
        with mock.patch.object(r, "load_table", lambda: table):
            with self.assertRaises(SystemExit) as ctx:
                r.main([])
        msg = str(ctx.exception)
        self.assertIn("no page uses eqty_sdk.extra", msg)
        self.assertIn("remove the entry from scripts/api-directives.json", msg)
        self.assertEqual(self.page.read_text(), before)

    def test_a_broken_page_is_named(self) -> None:
        (self.dir / "broken.mdx").write_text("prose\n{/* end generated */}\n")
        with self.assertRaises(SystemExit) as ctx:
            r.main([])
        self.assertIn("broken.mdx", str(ctx.exception))

    def test_a_broken_page_leaves_every_page_as_it_was(self) -> None:
        before = self.page.read_text()
        (self.dir / "q.mdx").write_text("prose\n{/* end generated */}\n")
        with self.assertRaises(SystemExit):
            r.main([])
        self.assertEqual(self.page.read_text(), before)


def fake_package(test: unittest.TestCase, files: dict[str, str]):
    """Load a throwaway package from `files` (path: source); removed after the test."""
    tmp = tempfile.TemporaryDirectory()
    test.addCleanup(tmp.cleanup)
    for rel, source in files.items():
        path = Path(tmp.name) / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(source)
    return r.griffe.load("fake", search_paths=[tmp.name], allow_inspection=False)


class RenderAdvice(unittest.TestCase):
    """Names the stub no longer has, as after `generate_stubs.py` rebuilds it."""

    def test_a_re_export_of_a_renamed_name_says_to_fix_the_import(self) -> None:
        pkg = fake_package(
            self,
            {
                "fake/__init__.py": "from fake._impl import gone\n",
                "fake/_impl.py": "def renamed() -> None: ...\n",
            },
        )
        with self.assertRaises(SystemExit) as ctx:
            r.render(pkg, "fake.gone", {}, advise=True)
        msg = str(ctx.exception)
        self.assertIn("Could not resolve alias fake.gone", msg)
        self.assertIn("update that import to the new name, then run `just generate-stubs`", msg)

    def test_a_path_through_a_renamed_re_export_says_to_fix_the_import(self) -> None:
        pkg = fake_package(
            self,
            {
                "fake/__init__.py": "from fake._impl import Gone\n",
                "fake/_impl.py": "class Renamed:\n    def load(self) -> None: ...\n",
            },
        )
        with self.assertRaises(SystemExit) as ctx:
            r.render(pkg, "fake.Gone.load", {}, advise=True)
        msg = str(ctx.exception)
        self.assertIn("Could not resolve alias fake.Gone", msg)
        self.assertIn("update that import to the new name, then run `just generate-stubs`", msg)

    def test_a_renamed_re_export_inside_a_module_fails(self) -> None:
        pkg = fake_package(
            self,
            {
                "fake/__init__.py": "",
                "fake/sub.py": (
                    "from fake._impl import Gone2\n\n"
                    "__all__ = ['Gone2', 'f']\n\n\n"
                    "def f() -> None:\n"
                    '    """Do f."""\n'
                ),
                "fake/_impl.py": "class Renamed2: ...\n",
            },
        )
        with self.assertRaises(SystemExit) as ctx:
            r.render(pkg, "fake.sub", {}, advise=True)
        msg = str(ctx.exception)
        self.assertIn("fake.sub.Gone2", msg)
        self.assertIn("update that import to the new name, then run `just generate-stubs`", msg)

    def test_imports_from_other_packages_are_not_checked(self) -> None:
        pkg = fake_package(
            self,
            {
                "fake/__init__.py": "",
                "fake/sub.py": (
                    "from elsewhere import Thing\n\n"
                    "__all__ = ['Thing', 'f']\n\n\n"
                    "def f() -> None:\n"
                    '    """Do f."""\n'
                ),
            },
        )
        self.assertIn("Do f.", r.render(pkg, "fake.sub", {}, advise=True))

    def test_a_missing_target_says_how_to_fix_the_docs(self) -> None:
        pkg = fake_package(self, {"fake/__init__.py": ""})
        with self.assertRaises(SystemExit) as ctx:
            r.render(pkg, "fake.nope", {}, advise=True)
        msg = str(ctx.exception)
        self.assertIn("fake.nope is not in fake", msg)
        self.assertIn("scripts/api-directives.json", msg)

    def test_old_releases_get_no_advice_about_this_repo(self) -> None:
        pkg = fake_package(
            self,
            {
                "fake/__init__.py": "from fake._impl import gone\n",
                "fake/_impl.py": "def renamed() -> None: ...\n",
            },
        )
        for target in ("fake.gone", "fake.nope"):
            with self.subTest(target=target):
                with self.assertRaises(SystemExit) as ctx:
                    r.render(pkg, target, {})
                self.assertNotIn("api-directives.json", str(ctx.exception))
                self.assertNotIn("update that import", str(ctx.exception))


class MissingRenderer(unittest.TestCase):
    def test_a_missing_griffe_keeps_the_error_and_says_to_run_just_install(self) -> None:
        script = Path(__file__).resolve().parent / "render_api_docs.py"
        code = (
            "import runpy, sys; sys.modules['griffe'] = None; "
            f"runpy.run_path({str(script)!r}, run_name='__main__')"
        )
        out = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True)
        self.assertEqual(out.returncode, 1)
        self.assertIn("import of griffe halted", out.stderr)
        self.assertIn("Run `just install`", out.stderr)


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
