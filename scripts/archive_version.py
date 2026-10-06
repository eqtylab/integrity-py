"""Turn an old release's MkDocs docs into an archived version of the new docs site.

    .venv-docs/bin/python scripts/archive_version.py v2.2.0 docs-site/archive/v2.2

Everything but the wheel reports comes from the release tag, never from the working tree: the
pages in docs/, mkdocs.yml, the files the pages include, and the eqty_sdk source. So the API
reference describes that release, not the current code.

Each page gets the changes the current pages got when they moved to the new site:

- its title is its label in mkdocs.yml's nav, and its own `# Heading` is removed
- `--8<--` snippet includes are replaced by the files they include
- links to .md pages point at the .mdx pages instead
- `:::` directives are replaced by the API reference, rendered with griffe2md

Wheel reports are the exception: a tag only has placeholders, so the real reports come from
the old published site (see GH_PAGES). A backport released after that site stopped has none
there, so archive_release.sh passes `--reports` with the four made from its published wheels.
Each folder also gets a `_group.yaml`, so the sidebar keeps the old nav's labels and order.

The pages have no `{/* generated */}` markers, so render_api_docs.py never refills them with
the current API. The output is committed, and running the script again on the same tag
writes the same files.
"""

from __future__ import annotations

import argparse
import html as htmllib
import io
import json
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path
from typing import Callable

import griffe
import griffe2md
import render_api_docs as r
import yaml

# mkdocstrings options griffe2md doesn't take. Dropping them changes nothing: paths and
# find_stubs_package say where to load the package from, which load_package handles here;
# show_source is off in every old release; show_root_toc_entry only affected MkDocs' table of
# contents.
IGNORED = {"show_root_toc_entry", "show_source", "find_stubs_package", "paths"}
# The old pages were rendered by mkdocstrings, whose defaults for these three differ from
# griffe2md's, and no old mkdocs.yml sets them. With griffe2md's defaults the reference would
# add every submodule, every inherited member and a summary of modules the page never shows.
MKDOCSTRINGS = {"show_submodules": False, "inherited_members": False, "summary": False}
# The `# Heading` a page starts with, and the blank lines after it.
H1 = re.compile(r"\A# .+\n+")
# A code fence whose only line is a snippet include: ```lang, --8<-- "path", ```.
FENCED_SNIPPET = re.compile(r'^```(\w*)\n--8<-- "([^"]+)"\n```$', re.M)
# A snippet include on a line of its own, outside a fence.
SNIPPET = re.compile(r'^--8<-- "([^"]+)"$', re.M)
# A relative link to a .md page, with its #anchor if any. URLs, #anchors and /paths are skipped.
MD_LINK = re.compile(r"\]\((?!https?:|#|/)([^)#\s]+)\.md(#[^)\s]*)?\)")
# A `::: target` line and the indented options block under it.
DIRECTIVE = re.compile(r"^::: (\S+)\n((?:[ \t]+.*\n|\n(?=[ \t]))*)", re.M)
# The space after a sentence ends, used to cut the description down to the first sentence.
SENTENCE = re.compile(r"(?<=[.!?])\s")
# The old MkDocs site's last commit on the gh-pages branch. It's the only place the real wheel
# reports survive: release CI generated them after tagging and published them without
# committing them, so every tag has placeholder reports.
GH_PAGES = "cb7be3c"
FENCE = re.compile(r"^```.*?^```$", re.M | re.S)
PRE = re.compile(r"(?s)<pre[^>]*>(.*?)</pre>")
# Snippet paths that are wheel reports.
REPORT_PATH = re.compile(r"^docs/generated/[^/]+\.txt$")
COMMENT = re.compile(r"<!--.*?-->", re.S)
NOT_ARCHIVED = "The wheel report for this release was not archived."


def parse_directive_options(block: str) -> dict:
    """The `options:` of a `:::` directive, read from the indented block under it."""
    if not block.strip():
        return {}
    data = yaml.safe_load("\n".join(line[4:] for line in block.splitlines()))
    return (data or {}).get("options", {}) or {}


def renderer_options(options: dict) -> dict:
    """The options to pass griffe2md: the given ones, minus IGNORED.

    An option this script doesn't know fails the run, so a setting the old site relied on is
    never dropped without anyone noticing.
    """
    unknown = sorted(set(options) - set(griffe2md.default_config) - IGNORED)
    if unknown:
        raise SystemExit(f"archive_version: unknown mkdocstrings option {', '.join(unknown)}")
    return {k: v for k, v in options.items() if k not in IGNORED}


def nav_index(nav: list) -> tuple[dict[str, str], dict[str, str]]:
    """Read the old mkdocs.yml nav: each page's title, and each folder's `_group.yaml`.

    A page's title is its nav label. A folder's `_group.yaml` gives the new site's sidebar the
    folder's old label and page order. The "" entry orders the top level.
    """
    titles: dict[str, str] = {}
    groups: dict[str, str] = {}
    root_order: list[str] = []
    for item in nav:
        ((label, value),) = item.items()
        if isinstance(value, str):
            titles[value] = label
            if value != "index.md":
                root_order.append(Path(value).stem)
            continue
        folder = Path(next(iter(value[0].values()))).parent.as_posix()
        order, index_label = [], None
        for child in value:
            ((child_label, path),) = child.items()
            titles[path] = child_label
            if Path(path).name == "index.md":
                index_label = child_label
            else:
                order.append(Path(path).stem)
        root_order.append(folder)
        lines = [f"label: {label}"]
        if index_label:
            lines.append(f"indexLabel: {index_label}")
        lines += ["order:"] + [f"  - {o}" for o in order]
        groups[folder] = "\n".join(lines) + "\n"
    groups[""] = "order:\n" + "".join(f"  - {o}\n" for o in root_order)
    return titles, groups


def report_blocks(page_html: str) -> list[str]:
    """The text of each <pre> block on an old MkDocs page, in page order."""
    return [htmllib.unescape(re.sub(r"<[^>]+>", "", m)).strip("\n") for m in PRE.findall(page_html)]


def old_page_html(tag: str, rel_md: str) -> str | None:
    """The old site's HTML for one page of this release, or None if the old site has no copy."""
    rel = f"{tag.lstrip('v')}/{Path(rel_md).with_suffix('.html').as_posix()}"
    out = subprocess.run(
        ["git", "show", f"{GH_PAGES}:{rel}"], cwd=r.ROOT, capture_output=True, text=True
    )
    return out.stdout if out.returncode == 0 else None


def convert_page(
    md: str,
    title: str,
    read: Callable[[str], str],
    api: dict[str, str],
    reports: list[str] | None = None,
) -> str:
    """Convert one old Markdown page into an MDX page for the new site.

    `read` returns a file for a snippet include: from the tag, or a wheel report from
    --reports. `api` holds the rendered reference for each directive's target. `reports` is
    the old page's <pre> blocks, used for wheel reports; it's None when the page has none, or
    when `read` supplies them.
    """
    body = H1.sub("", md, count=1)
    # Wheel reports come from the old published page. It has one <pre> per code block, in the
    # same order as the Markdown's fences, so a report's fence takes the <pre> at its position.
    if reports is not None and any(
        REPORT_PATH.match(m.group(2)) for m in FENCED_SNIPPET.finditer(body)
    ):
        fences = list(FENCE.finditer(body))
        if reports and len(reports) != len(fences):
            raise SystemExit(
                f"archive_version: {len(fences)} code blocks but {len(reports)} on the old page"
            )

        def report(m: re.Match) -> str:
            if not REPORT_PATH.match(m.group(2)):
                return m.group(0)
            if not reports:
                return NOT_ARCHIVED
            index = next(i for i, f in enumerate(fences) if f.start() == m.start())
            return r.fence(reports[index], m.group(1)).rstrip()

        body = FENCED_SNIPPET.sub(report, body)
    body = FENCED_SNIPPET.sub(lambda m: r.fence(read(m.group(2)), m.group(1)).rstrip(), body)
    body = SNIPPET.sub(lambda m: read(m.group(1)).rstrip(), body)
    body = MD_LINK.sub(
        lambda m: (
            f"]({'' if m.group(1).startswith('.') else './'}{m.group(1)}.mdx{m.group(2) or ''})"
        ),
        body,
    )
    body = DIRECTIVE.sub(lambda m: api[m.group(1)].rstrip() + "\n\n", body)
    # Remove HTML comments outside code blocks: MkDocs never showed them, and MDX can't parse
    # them. A comment inside a code block is part of the example, so it stays.
    parts, last = [], 0
    for f in FENCE.finditer(body):
        parts += [COMMENT.sub("", body[last : f.start()]), f.group(0)]
        last = f.end()
    body = "".join(parts) + COMMENT.sub("", body[last:])
    body = re.sub(r"\n{3,}", "\n\n", body).strip() + "\n"
    # The description is the first sentence of the first paragraph of prose: not a heading,
    # code block, list or table.
    first = next(p for p in body.split("\n\n") if p and not p.startswith(("#", "```", "-", "|")))
    description = SENTENCE.split(" ".join(first.split()), maxsplit=1)[0]
    return f"---\ntitle: {json.dumps(title)}\ndescription: {json.dumps(description)}\n---\n\n{body}"


def extract(tag: str, dest: Path) -> None:
    """Write the files of `tag` into `dest`."""
    data = subprocess.run(
        ["git", "archive", tag], cwd=r.ROOT, check=True, capture_output=True
    ).stdout
    with tarfile.open(fileobj=io.BytesIO(data)) as tar:
        # No extraction filter: the archive is our own tag, and the Python 3.9 in .venv-docs
        # has no filter argument.
        tar.extractall(dest)


def load_package(tree: Path) -> griffe.Module:
    # Load eqty_sdk from the extracted tag only. By default griffe first looks for an eqty_sdk
    # folder in the working directory, which from the repo root is the current package.
    return griffe.load(
        "eqty_sdk",
        search_paths=[str(tree)],
        try_relative_path=False,
        find_stubs_package=True,
        allow_inspection=False,
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    parser.add_argument("tag")
    parser.add_argument("out", type=Path)
    parser.add_argument(
        "--reports",
        type=Path,
        help="a folder holding the release's wheel reports, read in place of the old site's",
    )
    args = parser.parse_args(argv)

    with tempfile.TemporaryDirectory() as tmp:
        tree = Path(tmp)
        extract(args.tag, tree)
        mkdocs = yaml.safe_load((tree / "mkdocs.yml").read_text())
        # Rendering options, later ones winning: this site's (render_api_docs.DEFAULTS), then
        # mkdocstrings' defaults where griffe2md's differ, then the old mkdocs.yml's own.
        handler = next(
            p["mkdocstrings"]
            for p in mkdocs["plugins"]
            if isinstance(p, dict) and "mkdocstrings" in p
        )
        defaults = {
            **r.DEFAULTS,
            **MKDOCSTRINGS,
            **renderer_options(handler["handlers"]["python"].get("options", {})),
        }
        titles, groups = nav_index(mkdocs["nav"])

        def read(rel: str) -> str:
            """A file from the tag, or a wheel report from --reports, for a snippet include. A
            missing file fails the run.
            """
            if args.reports and REPORT_PATH.match(rel):
                path = args.reports / Path(rel).name
                if not path.is_file():
                    raise SystemExit(f"archive_version: {path} is missing")
                return path.read_text()
            path = tree / rel
            if not path.is_file():
                raise SystemExit(f"archive_version: {args.tag} has no {rel}")
            return path.read_text()

        # Every page in the tag's docs/. docs/generated/ holds files pages include, not pages.
        pages = {
            p.relative_to(tree / "docs").as_posix(): p.read_text()
            for p in sorted((tree / "docs").rglob("*.md"))
            if not p.relative_to(tree / "docs").as_posix().startswith("generated/")
        }
        # Render every directive before making the Markdown MDX-safe, as render_api_docs.py
        # does: that step points links at heading ids, so it needs all the headings first.
        pkg = load_package(tree)
        rendered = {}
        for md in pages.values():
            for m in DIRECTIVE.finditer(md):
                opts = renderer_options(parse_directive_options(m.group(2)))
                rendered[m.group(1)] = r.render(pkg, m.group(1), opts, defaults=defaults)
        heading_ids = {
            h.group(1): r.slug(h.group(1))
            for md in rendered.values()
            for h in r.HEADING.finditer(md)
        }
        api = {t: r.mdx_safe(md, heading_ids) for t, md in rendered.items()}

        # Start from an empty folder, so no page from an earlier run is left behind.
        if args.out.exists():
            shutil.rmtree(args.out)
        for rel, md in pages.items():
            if rel not in titles:
                raise SystemExit(f"archive_version: {rel} is not in {args.tag}'s nav")
            dest = args.out / Path(rel).with_suffix(".mdx")
            dest.parent.mkdir(parents=True, exist_ok=True)
            # A page that includes wheel reports takes them from --reports, through read(), or
            # else from the old site's copy of it.
            reports = None
            if not args.reports and any(
                REPORT_PATH.match(m.group(2)) for m in FENCED_SNIPPET.finditer(md)
            ):
                page = old_page_html(args.tag, rel)
                reports = report_blocks(page) if page else []
            dest.write_text(convert_page(md, titles[rel], read, api, reports=reports))
        # The sidebar's labels and order, from the old nav.
        for folder, text in groups.items():
            (args.out / folder / "_group.yaml").write_text(text)
    print(f"archive_version: {args.tag} → {args.out}, {len(pages)} pages, {len(api)} API blocks")
    return 0


if __name__ == "__main__":
    sys.exit(main())
