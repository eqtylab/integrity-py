"""Tests for archive_version.py.

    .venv-docs/bin/python -m unittest scripts/test_archive_version.py

The RealTag tests convert real release tags, so they need the repo's tags.
"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import archive_version as a  # noqa: E402


class Pages(unittest.TestCase):
    def convert(self, md, api=None, files=None):
        files = files or {}
        return a.convert_page(md, "Signer", lambda p: files[p], api or {})

    def test_title_from_nav_and_h1_dropped_description_first_sentence(self) -> None:
        out = self.convert("# Old Title\n\nFirst sentence. Second one.\n\nMore.\n")
        self.assertTrue(
            out.startswith(
                '---\ntitle: "Signer"\ndescription: "First sentence."\n---\n\nFirst sentence.'
            )
        )
        self.assertNotIn("# Old Title", out)

    def test_md_links_become_mdx_and_keep_anchors(self) -> None:
        out = self.convert(
            "# T\n\nSee [a](signer.md), [b](../api/assets.md#built-in), [c](https://x.io/a.md).\n"
        )
        self.assertIn("[a](./signer.mdx)", out)
        self.assertIn("[b](../api/assets.mdx#built-in)", out)
        self.assertIn("[c](https://x.io/a.md)", out)

    def test_fenced_snippet_is_inlined_in_its_fence(self) -> None:
        out = self.convert(
            '# T\n\nx\n\n```python\n--8<-- "examples/a.py"\n```\n',
            files={"examples/a.py": "x = 1\n"},
        )
        self.assertIn("```python\nx = 1\n```", out)

    def test_bare_snippet_is_inlined_as_markdown(self) -> None:
        out = self.convert(
            '# T\n\nx\n\n--8<-- "docs/generated/list.md"\n',
            files={"docs/generated/list.md": "- one\n"},
        )
        self.assertIn("\n- one\n", out)

    def test_directive_is_replaced_by_its_rendered_block(self) -> None:
        md = "# T\n\nx\n\n::: eqty_sdk._rust.Signer\n    options:\n      members_order: source\n\n## Next\n"
        out = self.convert(md, api={"eqty_sdk._rust.Signer": "### `Signer`\n"})
        self.assertIn("\n### `Signer`\n\n## Next", out)
        self.assertNotIn(":::", out)

    def test_report_snippets_take_the_old_site_text_in_order(self) -> None:
        md = (
            "# T\n\nx\n\n```bash\npip install eqty_sdk\n```\n\n"
            '```text\n--8<-- "docs/generated/a.txt"\n```\n\n```text\n--8<-- "docs/generated/b.txt"\n```\n'
        )
        html = "<pre><code>pip install eqty_sdk</code></pre><pre><code>real &amp; one</code></pre><pre>real two</pre>"
        out = a.convert_page(
            md, "Min", lambda p: "placeholder\n", {}, reports=a.report_blocks(html)
        )
        self.assertIn("```text\nreal & one\n```", out)
        self.assertIn("```text\nreal two\n```", out)
        self.assertNotIn("placeholder", out)

    def test_report_snippets_without_an_old_page_become_a_note(self) -> None:
        md = '# T\n\nx\n\n```text\n--8<-- "docs/generated/a.txt"\n```\n'
        out = a.convert_page(md, "Min", lambda p: "placeholder\n", {}, reports=[])
        self.assertIn("The wheel report for this release was not archived.", out)

    def test_report_block_count_mismatch_fails(self) -> None:
        md = '# T\n\nx\n\n```text\n--8<-- "docs/generated/a.txt"\n```\n'
        with self.assertRaises(SystemExit):
            a.convert_page(md, "Min", str, {}, reports=["one", "two"])

    def test_html_comments_are_dropped_outside_code(self) -> None:
        # MkDocs never showed HTML comments, and MDX can't parse them.
        out = self.convert(
            "# T\n\nx\n\n<!-- TODO: a\n     b -->\n\ny\n\n```html\n<!-- kept -->\n```\n"
        )
        self.assertNotIn("TODO", out)
        self.assertIn("x\n\ny", out)
        self.assertIn("<!-- kept -->", out)


class Options(unittest.TestCase):
    def test_options_block_parses(self) -> None:
        block = "    options:\n      members:\n        - ED25519\n      members_order: source\n"
        self.assertEqual(
            a.parse_directive_options(block), {"members": ["ED25519"], "members_order": "source"}
        )

    def test_toc_only_options_are_dropped(self) -> None:
        self.assertEqual(
            a.renderer_options({"show_root_toc_entry": False, "heading_level": 3}),
            {"heading_level": 3},
        )

    def test_unknown_option_fails_naming_it(self) -> None:
        with self.assertRaises(SystemExit) as ctx:
            a.renderer_options({"show_bogus": True})
        self.assertIn("show_bogus", str(ctx.exception))


class Nav(unittest.TestCase):
    NAV = [
        {"Home": "index.md"},
        {"Install": [{"Min Version": "install/min-version.md"}, {"Source": "install/source.md"}]},
        {"API Reference": [{"Overview": "api/index.md"}, {"Signer": "api/signer.md"}]},
    ]

    def test_titles_and_groups(self) -> None:
        titles, groups = a.nav_index(self.NAV)
        self.assertEqual(titles["install/source.md"], "Source")
        self.assertEqual(titles["api/index.md"], "Overview")
        self.assertEqual(groups[""], "order:\n  - install\n  - api\n")
        self.assertEqual(groups["install"], "label: Install\norder:\n  - min-version\n  - source\n")
        self.assertEqual(
            groups["api"], "label: API Reference\nindexLabel: Overview\norder:\n  - signer\n"
        )


class RealTag(unittest.TestCase):
    def test_package_is_the_tags_not_the_working_trees(self) -> None:
        # Runs from the repo root, which has the current eqty_sdk. v2.0.9 had an Attribution
        # asset that has since been removed, so finding it proves the package came from the tag.
        import tempfile

        tree = Path(tempfile.mkdtemp())
        a.extract("v2.0.9", tree)
        pkg = a.load_package(tree)
        self.assertEqual(Path(pkg.filepath).resolve().parent.parent, tree.resolve())
        self.assertIn("Attribution", pkg["asset"].members)

    def test_v2_2_0_converts_every_page_and_is_stable(self) -> None:
        import filecmp
        import tempfile

        one, two = Path(tempfile.mkdtemp()), Path(tempfile.mkdtemp())
        a.main(["v2.2.0", str(one)])
        a.main(["v2.2.0", str(two)])
        pages = sorted(p.relative_to(one) for p in one.rglob("*.mdx"))
        self.assertEqual(len(pages), 27)
        cmp = filecmp.dircmp(one, two)
        self.assertEqual((cmp.diff_files, cmp.left_only, cmp.right_only), ([], [], []))
        text = "".join(p.read_text() for p in one.rglob("*.mdx"))
        for leftover in (":::", "--8<--", ".md)", ".md#"):
            self.assertNotIn(leftover, text)
        # Rendered with mkdocstrings' defaults, as the old site was: the package's classes, with
        # no submodules (such as skill) and no inherited members (such as __add__).
        assets = (one / "api/assets.mdx").read_text()
        self.assertNotIn("eqty_sdk.asset.skill", assets)
        self.assertNotIn("__add__", assets)
        self.assertIn("Agent`", assets)

    def test_reports_given_by_hand_replace_the_old_site_s(self) -> None:
        # A backport released after the old site stopped has no page on cb7be3c to read from.
        import tempfile

        reports, out = Path(tempfile.mkdtemp()), Path(tempfile.mkdtemp())
        for name in REPORTS:
            (reports / name).write_text(f"by hand: {name}\n")
        a.main(["v2.4.2", str(out), "--reports", str(reports)])
        page = (out / "install/min-version.mdx").read_text()
        for name in REPORTS:
            self.assertIn(f"```text\nby hand: {name}\n```", page)
        self.assertNotIn("manylinux_2_17_x86_64", page)

    def test_a_missing_report_given_by_hand_fails_naming_it(self) -> None:
        import tempfile

        reports, out = Path(tempfile.mkdtemp()), Path(tempfile.mkdtemp())
        for name in REPORTS[1:]:
            (reports / name).write_text("by hand\n")
        with self.assertRaises(SystemExit) as ctx:
            a.main(["v2.4.2", str(out), "--reports", str(reports)])
        self.assertIn(REPORTS[0], str(ctx.exception))


REPORTS = [
    "auditwheel-show-linux-x86_64.txt",
    "auditwheel-show-linux-aarch64.txt",
    "otool-show-macos-arm64.txt",
    "otool-show-macos-x86_64.txt",
]
