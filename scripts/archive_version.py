"""Convert one old release's MkDocs docs into a frozen version of the new site.

    .venv-docs/bin/python scripts/archive_version.py v2.2.0 docs-site/archive/v2.2

Everything is read from the tag, never the working tree: the pages, mkdocs.yml, the files
they include, and eqty_sdk as released, so the API reference is that release's. The four
conversions are the ones #91 made by hand for the current pages: the nav label becomes the
title and the H1 goes, snippet includes are inlined, .md links point at .mdx pages, and
`:::` directives become the rendered reference. The output has no generated markers, so
render_api_docs.py never rewrites it with today's code. It is committed; a second run on the
same tag writes identical files.
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

# Options mkdocstrings reads that change nothing in the rendered Markdown.
IGNORED = {"show_root_toc_entry", "show_source", "find_stubs_package", "paths"}
# mkdocstrings' own defaults where griffe2md's differ and the old mkdocs.yml set nothing. The old
# pages were rendered with these; griffe2md's would add every submodule, inherited member and a
# summary list linking to modules the page never shows.
MKDOCSTRINGS = {"show_submodules": False, "inherited_members": False, "summary": False}
H1 = re.compile(r"\A# .+\n+")
FENCED_SNIPPET = re.compile(r'^```(\w*)\n--8<-- "([^"]+)"\n```$', re.M)
SNIPPET = re.compile(r'^--8<-- "([^"]+)"$', re.M)
MD_LINK = re.compile(r"\]\((?!https?:|#|/)([^)#\s]+)\.md(#[^)\s]*)?\)")
DIRECTIVE = re.compile(r"^::: (\S+)\n((?:[ \t]+.*\n|\n(?=[ \t]))*)", re.M)
SENTENCE = re.compile(r"(?<=[.!?])\s")
# The last gh-pages commit of the old MkDocs site. Release CI wrote each release's wheel reports
# just before mike deployed, so its pages are the only copy; the tag holds placeholders.
GH_PAGES = "cb7be3c"
FENCE = re.compile(r"^```.*?^```$", re.M | re.S)
PRE = re.compile(r"(?s)<pre[^>]*>(.*?)</pre>")
REPORT_PATH = re.compile(r"^docs/generated/[^/]+\.txt$")
COMMENT = re.compile(r"<!--.*?-->", re.S)
NOT_ARCHIVED = "The wheel report for this release was not archived."


def parse_directive_options(block: str) -> dict:
    if not block.strip():
        return {}
    data = yaml.safe_load("\n".join(line[4:] for line in block.splitlines()))
    return (data or {}).get("options", {}) or {}


def renderer_options(options: dict) -> dict:
    unknown = sorted(set(options) - set(griffe2md.default_config) - IGNORED)
    if unknown:
        raise SystemExit(f"archive_version: unknown mkdocstrings option {', '.join(unknown)}")
    return {k: v for k, v in options.items() if k not in IGNORED}


def nav_index(nav: list) -> tuple[dict[str, str], dict[str, str]]:
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
    """Every <pre> block of an old MkDocs page, as text, in page order."""
    return [htmllib.unescape(re.sub(r"<[^>]+>", "", m)).strip("\n") for m in PRE.findall(page_html)]


def old_page_html(tag: str, rel_md: str) -> str | None:
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
    body = H1.sub("", md, count=1)
    # The old page has one <pre> per fenced block, in the same order, so a report snippet takes
    # the old page's block at its own position.
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
    # MkDocs never showed HTML comments, and MDX rejects them. Code blocks keep theirs.
    parts, last = [], 0
    for f in FENCE.finditer(body):
        parts += [COMMENT.sub("", body[last : f.start()]), f.group(0)]
        last = f.end()
    body = "".join(parts) + COMMENT.sub("", body[last:])
    body = re.sub(r"\n{3,}", "\n\n", body).strip() + "\n"
    first = next(p for p in body.split("\n\n") if p and not p.startswith(("#", "```", "-", "|")))
    description = SENTENCE.split(" ".join(first.split()), maxsplit=1)[0]
    return f"---\ntitle: {json.dumps(title)}\ndescription: {json.dumps(description)}\n---\n\n{body}"


def extract(tag: str, dest: Path) -> None:
    data = subprocess.run(
        ["git", "archive", tag], cwd=r.ROOT, check=True, capture_output=True
    ).stdout
    with tarfile.open(fileobj=io.BytesIO(data)) as tar:
        tar.extractall(dest)  # our own tag; Python 3.9 has no filter argument


def load_package(tree: Path) -> griffe.Module:
    # Without try_relative_path=False, griffe first tries "eqty_sdk" as a path from the working
    # directory, and from the repo root that is today's package, not the tag's.
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
    args = parser.parse_args(argv)

    with tempfile.TemporaryDirectory() as tmp:
        tree = Path(tmp)
        extract(args.tag, tree)
        mkdocs = yaml.safe_load((tree / "mkdocs.yml").read_text())
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
            path = tree / rel
            if not path.is_file():
                raise SystemExit(f"archive_version: {args.tag} has no {rel}")
            return path.read_text()

        pages = {
            p.relative_to(tree / "docs").as_posix(): p.read_text()
            for p in sorted((tree / "docs").rglob("*.md"))
            if not p.relative_to(tree / "docs").as_posix().startswith("generated/")
        }
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

        if args.out.exists():
            shutil.rmtree(args.out)
        for rel, md in pages.items():
            if rel not in titles:
                raise SystemExit(f"archive_version: {rel} is not in {args.tag}'s nav")
            dest = args.out / Path(rel).with_suffix(".mdx")
            dest.parent.mkdir(parents=True, exist_ok=True)
            if any(REPORT_PATH.match(m.group(2)) for m in FENCED_SNIPPET.finditer(md)):
                page = old_page_html(args.tag, rel)
                reports = report_blocks(page) if page else []
            else:
                reports = None
            dest.write_text(convert_page(md, titles[rel], read, api, reports=reports))
        for folder, text in groups.items():
            (args.out / folder / "_group.yaml").write_text(text)
    print(f"archive_version: {args.tag} → {args.out}, {len(pages)} pages, {len(api)} API blocks")
    return 0


if __name__ == "__main__":
    sys.exit(main())
