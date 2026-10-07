"""Fill the docs pages' generated blocks: the API reference, via griffe2md, and repo files.

    poetry run python scripts/render_api_docs.py [--check]

`just generate-stubs`, `just serve-docs` and `just build-docs` run it; `just install` puts
griffe and griffe2md in the Poetry env. griffe reads eqty_sdk/_rust.pyi statically
(find_stubs_package), so no compiled extension is needed; `just generate-stubs` rebuilds that
stub from the Rust doc comments first. Output is committed, so the site build needs Node alone.

Output is written into the pages under docs-site/src/content/docs/, between marker comments:

    {/* generated api eqty_sdk._rust.Signer */}      rendered from scripts/api-directives.json
    {/* generated file examples/quick-start.py python */}   a repo file, fenced
    {/* generated file docs/generated/built-in-asset-types.md */}   a repo file, as Markdown
    {/* end generated */}

The content is in the page, not imported, because @eqtylab/docs builds each older version
from its saved copy under docs-site/archive/ and strips every import, component and
expression from it. Anything imported would vanish from the archived copy. --check exits 1
when a page no longer matches what the script would write, and changes nothing.

griffe2md writes Markdown for MkDocs. These rewrites make it MDX and make it fit this site:
  1. `<https://x>` autolinks are JSX to MDX; they become `[x](x)`.
  2. `<code>[Name](#anchor)</code>` type cross-references, one or several to a code span,
     point at anchors MkDocs would have made; here they become plain `Name` in code, since a
     type with no heading in these partials has nowhere to go.
  3. In-partial links `(#eqty_sdk._rust.Signer.load)` are rewritten to the id the site gives
     that heading. @eqtylab/docs uses github-slugger over the heading text; for these
     headings (letters, digits, underscores, dots) that is lowercase with the dots removed.
  4. Signatures: see fix_signatures.
  5. `<code>X</code>` spans become `X` in backticks; MDX parses the tag as JSX, which archives
     strip.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Callable

try:
    import griffe
    import griffe2md
except ImportError as err:  # an env set up before griffe joined the Poetry dev group
    raise SystemExit(
        f"render_api_docs: {err.name} is not installed. Run `just install`, then try again."
    ) from err

ROOT = Path(__file__).resolve().parent.parent
TABLE = ROOT / "scripts" / "api-directives.json"
CONTENT = ROOT / "docs-site" / "src" / "content" / "docs"

# How to fix the docs when a reference target has left the code. render() leaves it out,
# since archive_version.py renders old releases, where it would be wrong.
RENAMED = (
    "If it was renamed or removed, update its entry in scripts/api-directives.json and the "
    "{/* generated api ... */} marker in the page that shows it."
)

# The options every directive starts from; a directive's own options override them.
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

HEADING = re.compile(r"^#{1,6} `([^`]+)`\s*$", re.M)
AUTOLINK = re.compile(r"<(https?://[^>\s]+)>")
CODE_XREF = re.compile(r"<code>\[([^\]]+)\]\(#[^)]*\)</code>")
# A code span can hold several cross-references, as in `Optional[UUID]`.
CODE_SPAN = re.compile(r"<code>.*?</code>")
LINK_IN_CODE = re.compile(r"\[([^\]]+)\]\(#[^)]*\)")
ANCHOR = re.compile(r"\]\(#([^)\s]+)\)")
BLOCK = re.compile(
    r"^\{/\* generated (?P<kind>api|file) (?P<key>\S+)(?: (?P<lang>\w+))? \*/\}\n"
    r"(?P<body>.*?)"
    r"^\{/\* end generated \*/\}$",
    re.M | re.S,
)
OPEN = re.compile(r"^\{/\* generated ", re.M)
END = re.compile(r"^\{/\* end generated \*/\}$", re.M)
CODE_TAG = re.compile(r"<code>(.*?)</code>")
MD_ESCAPE = re.compile(r"\\([\[\]_*])")


def slug(text: str) -> str:
    """github-slugger for the character set these headings use."""
    return re.sub(r"[^a-z0-9_ -]", "", text.lower()).replace(" ", "-")


def mdx_safe(markdown: str, heading_ids: dict[str, str]) -> str:
    markdown = AUTOLINK.sub(r"[\1](\1)", markdown)
    markdown = CODE_XREF.sub(r"`\1`", markdown)
    markdown = CODE_SPAN.sub(lambda m: LINK_IN_CODE.sub(r"\1", m.group(0)), markdown)
    markdown = CODE_TAG.sub(lambda m: "`" + MD_ESCAPE.sub(r"\1", m.group(1)) + "`", markdown)
    return ANCHOR.sub(
        lambda m: f"](#{heading_ids[m.group(1)]})" if m.group(1) in heading_ids else m.group(0),
        markdown,
    )


def fence(code: str, lang: str) -> str:
    runs = [len(m) for m in re.findall(r"`+", code)]
    ticks = "`" * max(3, max(runs, default=0) + 1)
    return f"{ticks}{lang}\n{code.rstrip()}\n{ticks}\n"


def fill(page: str, api: dict[str, str], read: Callable[[str], str]) -> tuple[str, set[str]]:
    used: set[str] = set()

    def body(m: re.Match) -> str:
        kind, key, lang = m.group("kind"), m.group("key"), m.group("lang")
        if kind == "api":
            if key not in api:
                raise SystemExit(
                    f"render_api_docs: a page shows {key}, which is not in "
                    "scripts/api-directives.json. Add it there, or fix the page's marker."
                )
            used.add(key)
            content = api[key]
        else:
            text = read(key)
            content = fence(text, lang) if lang else text
        head = m.group(0).split("\n", 1)[0]
        return f"{head}\n{content.rstrip()}\n{{/* end generated */}}"

    # Every opener needs an end and every end an opener; a stray one would otherwise pass.
    blocks = len(BLOCK.findall(page))
    if len(OPEN.findall(page)) != blocks or len(END.findall(page)) != blocks:
        raise SystemExit(
            "render_api_docs: a {/* generated */} marker has no partner. Each "
            "{/* generated ... */} line needs a {/* end generated */} line after it."
        )
    return BLOCK.sub(body, page), used


def read_repo_file(rel: str) -> str:
    path = ROOT / rel
    if not path.is_file():
        raise SystemExit(
            f"render_api_docs: {rel} does not exist. If it was renamed or moved, update the "
            "generated file marker that names it."
        )
    return path.read_text()


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


def render(pkg: griffe.Module, target: str, options: dict, defaults: dict | None = None) -> str:
    """The reference for `target` as griffe2md Markdown, with its signatures fixed.

    `options` are the directive's own and override `defaults`, which are DEFAULTS unless
    given. archive_version.py passes an old release's.
    """
    config = {**(DEFAULTS if defaults is None else defaults), **options}
    try:
        obj = pkg[target.split(".", 1)[1]]
    except KeyError as err:
        raise SystemExit(f"render_api_docs: {target} is not in eqty_sdk") from err
    return fix_signatures(obj, griffe2md.render_object_docs(obj, config)).rstrip() + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    parser.add_argument("--check", action="store_true", help="fail if any page would change")
    args = parser.parse_args(argv)

    pkg = load_package()
    try:
        rendered = {
            d["target"]: render(pkg, d["target"], d.get("options", {})) for d in load_table()
        }
    except SystemExit as err:  # render() names the missing target; add the docs fix
        raise SystemExit(f"{err}. {RENAMED}") from err
    # Two passes: every heading's id first, then the links that point at them.
    heading_ids = {
        m.group(1): slug(m.group(1)) for md in rendered.values() for m in HEADING.finditer(md)
    }
    api = {target: mdx_safe(md, heading_ids) for target, md in rendered.items()}

    used: set[str] = set()
    stale = []
    for page in sorted(CONTENT.rglob("*.mdx")):
        before = page.read_text()
        try:
            after, targets = fill(before, api, read_repo_file)
        except SystemExit as err:
            raise SystemExit(f"{err} Page: {page.relative_to(CONTENT)}") from err
        used |= targets
        if after != before:
            stale.append(page.relative_to(CONTENT))
            if not args.check:
                page.write_text(after)

    unused = sorted(set(api) - used)
    if unused:
        raise SystemExit(
            f"render_api_docs: no page uses {', '.join(unused)}. Add a "
            "{/* generated api ... */} marker to a page, or remove the entry from "
            "scripts/api-directives.json."
        )
    if args.check:
        for p in stale:
            print(f"out of date: {p}", file=sys.stderr)
        return 1 if stale else 0
    print(f"filled {len(used)} API blocks; rewrote {len(stale)} pages")
    return 0


if __name__ == "__main__":
    sys.exit(main())
