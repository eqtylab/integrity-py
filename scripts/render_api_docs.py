"""Render each mkdocstrings directive of the docs to an MDX partial with griffe2md.

    python3 -m venv .venv-docs && .venv-docs/bin/pip install griffe==1.14.0 griffe2md==1.2.5
    .venv-docs/bin/python scripts/render_api_docs.py

griffe reads eqty_sdk/_rust.pyi statically (find_stubs_package), so no compiled extension
is needed; run `just generate-stubs` first if the Rust doc comments changed. Output is
committed under docs-site/src/generated/, so the site build needs Node alone.

Every partial starts with a `components` export. @eqtylab/docs passes its component map
to the page's own content only; an imported .mdx partial renders bare <pre> and plain <a>
(observed in a real build, 2026-09-24). The export gives each partial the same bridges.

griffe2md writes Markdown for MkDocs. Three rewrites make it MDX and make it fit this site:
  1. `<https://x>` autolinks are JSX to MDX; they become `[x](x)`.
  2. `<code>[Name](#anchor)</code>` type cross-references, one or several to a code span,
     point at anchors MkDocs would have made; here they become plain `Name` in code, since a
     type with no heading in these partials has nowhere to go.
  3. In-partial links `(#eqty_sdk._rust.Signer.load)` are rewritten to the id the site gives
     that heading. @eqtylab/docs uses github-slugger over the heading text; for these
     headings (letters, digits, underscores, dots) that is lowercase with the dots removed.
"""

from __future__ import annotations

import json
import re
import shutil
from pathlib import Path

import griffe
import griffe2md

ROOT = Path(__file__).resolve().parent.parent
TABLE = ROOT / "scripts" / "api-directives.json"
OUT = ROOT / "docs-site" / "src" / "generated"
API_OUT = OUT / "api"

# The handler defaults from mkdocs.yml. Anything a directive does not override.
DEFAULTS = {
    **griffe2md.default_config,
    "filters": ["!^_"],
    "heading_level": 2,
    # griffe2md wraps any signature longer than this with Black whenever Black is importable,
    # which the Poetry dev group installs. One line per signature keeps the output the same
    # in every environment and keeps rewrite 4 on the line it expects.
    "line_length": 10_000,
    "members_order": "source",
    "merge_init_into_class": True,
    "separate_signature": True,
    "show_root_full_path": True,
    "show_root_heading": False,
    "show_root_members_full_path": True,
    "show_signature_annotations": True,
    "show_if_no_docstring": False,
}

PARTIAL_HEADER = """import CodeFenceBridge from '@eqtylab/docs/components/CodeFenceBridge.astro';
import Link from '@eqtylab/docs/components/Link.astro';

export const components = { pre: CodeFenceBridge, a: Link };

"""

HEADING = re.compile(r"^#{1,6} `([^`]+)`\s*$", re.M)
AUTOLINK = re.compile(r"<(https?://[^>\s]+)>")
CODE_XREF = re.compile(r"<code>\[([^\]]+)\]\(#[^)]*\)</code>")
# A code span can hold several cross-references, as in `Optional[UUID]`.
CODE_SPAN = re.compile(r"<code>.*?</code>")
LINK_IN_CODE = re.compile(r"\[([^\]]+)\]\(#[^)]*\)")
ANCHOR = re.compile(r"\]\(#([^)\s]+)\)")


def slug(text: str) -> str:
    """github-slugger for the character set these headings use."""
    return re.sub(r"[^a-z0-9_ -]", "", text.lower()).replace(" ", "-")


def mdx_safe(markdown: str, heading_ids: dict[str, str]) -> str:
    markdown = AUTOLINK.sub(r"[\1](\1)", markdown)
    markdown = CODE_XREF.sub(r"`\1`", markdown)
    markdown = CODE_SPAN.sub(lambda m: LINK_IN_CODE.sub(r"\1", m.group(0)), markdown)
    return ANCHOR.sub(
        lambda m: f"](#{heading_ids[m.group(1)]})" if m.group(1) in heading_ids else m.group(0),
        markdown,
    )


def _objects(obj: griffe.Object, root: str):
    """The object and every function or class under it, aliases resolved, staying inside
    `root` so that imports from other modules are not walked.
    """
    yield obj
    for member in obj.members.values():
        try:
            target = member.final_target if member.is_alias else member
        except griffe.AliasResolutionError:
            continue
        if not target.path.startswith(root):
            continue
        if target.is_function:
            yield target
        elif target.is_class or target.is_module:
            yield from _objects(target, root)


def _signature_params(o: griffe.Object) -> list:
    if o.is_class:
        init = o.members.get("__init__")
        return list(init.parameters) if init is not None and init.is_function else []
    return list(o.parameters) if o.is_function else []


def _strip_annotation(line: str, prefix: str) -> str:
    """Drop `: <annotation>` after `prefix` (`**kwargs`), up to a depth-0 `,` or `)`."""
    j = line.find(prefix + ": ")
    if j == -1:
        return line
    k, depth = j + len(prefix) + 2, 0
    while k < len(line) and not (depth == 0 and line[k] in ",)"):
        depth += line[k] in "[("
        depth -= line[k] in "])"
        k += 1
    return line[: j + len(prefix)] + line[k:]


def fix_signatures(obj: griffe.Object, markdown: str) -> str:
    """Rewrite 4. griffe2md's signature template carries the previous parameter's annotation
    onto an unannotated `*args` or `**kwargs`, and ends merged class signatures in `-> None`.
    Each signature line `name(...)` is matched to the griffe objects of that name under the
    target; which parameters are unannotated comes from griffe, never from the text.
    """
    by_name: dict[str, list[griffe.Object]] = {}
    for o in _objects(obj, obj.path):
        by_name.setdefault(o.name, []).append(o)

    def fix(line: str) -> str:
        m = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)\(", line)
        if not m or m.group(1) not in by_name:
            return line
        objs = by_name[m.group(1)]
        for kind, star in (("variadic keyword", "**"), ("variadic positional", "*")):
            params = [p for o in objs for p in _signature_params(o) if p.kind.value == kind]
            # Only when every same-named object agrees the parameter is unannotated.
            if (
                params
                and all(p.annotation is None for p in params)
                and len({p.name for p in params}) == 1
            ):
                line = _strip_annotation(line, star + params[0].name)
        if all(o.is_class for o in objs) and line.endswith(") -> None"):
            line = line[: -len(" -> None")]
        return line

    return "\n".join(fix(line) for line in markdown.split("\n"))


def load_package() -> griffe.Module:
    return griffe.load(
        "eqty_sdk", search_paths=[str(ROOT)], find_stubs_package=True, allow_inspection=False
    )


def load_table() -> list[dict]:
    return json.loads(TABLE.read_text())


def render(pkg: griffe.Module, target: str, options: dict) -> str:
    config = {**DEFAULTS, **options}
    try:
        obj = pkg[target.split(".", 1)[1]]
    except KeyError as err:
        raise SystemExit(f"render_api_docs: {target} is not in eqty_sdk") from err
    return fix_signatures(obj, griffe2md.render_object_docs(obj, config)).rstrip() + "\n"


def main() -> None:
    pkg = load_package()
    rendered = {d["target"]: render(pkg, d["target"], d.get("options", {})) for d in load_table()}

    # Two passes: every heading's id first, then the links that point at them.
    heading_ids = {
        m.group(1): slug(m.group(1)) for md in rendered.values() for m in HEADING.finditer(md)
    }

    if API_OUT.exists():
        shutil.rmtree(API_OUT)
    API_OUT.mkdir(parents=True)
    for target, md in rendered.items():
        (API_OUT / f"{target}.mdx").write_text(PARTIAL_HEADER + mdx_safe(md, heading_ids))

    # A Markdown bullet list the assets page includes inline. Copied as .mdx so a page can
    # import it as a component; the content is unchanged.
    src = ROOT / "docs" / "generated" / "built-in-asset-types.md"
    (OUT / "built-in-asset-types.mdx").write_text(src.read_text())

    print(f"rendered {len(rendered)} API partials into {API_OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
