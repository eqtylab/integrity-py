"""Every address the old MkDocs site served must end on a page of the new site, in its version.

    python3 scripts/check_old_links.py docs-site/dist            # rules against a local build
    python3 scripts/check_old_links.py https://<preview-url>     # real requests

The list is origin/gh-pages as it stands (commit cb7be3c), less the theme's 404.html pages,
plus the bare folder each index.html is served at and PyPI's Asset reference link, so it
covers every link anyone can hold. Locally, vercel.json's redirect rules are applied
as regexes; they are written with literal text and regex groups only so that this is exact.
The current release comes from this checkout's folders.json, so check a preview from the
branch it was built from.
"""

import json
import re
import subprocess
import sys
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Optional

ROOT = Path(__file__).resolve().parent.parent
GH_PAGES = "cb7be3c"


class _Follow308(urllib.request.HTTPRedirectHandler):
    """Vercel's permanent redirects are 308s, which urllib follows only from Python 3.11."""

    http_error_308 = urllib.request.HTTPRedirectHandler.http_error_302

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return super().redirect_request(req, fp, 307 if code == 308 else code, msg, headers, newurl)


OPENER = urllib.request.build_opener(_Follow308())


def old_addresses() -> list[str]:
    try:
        files = subprocess.run(
            ["git", "ls-tree", "-r", "--name-only", GH_PAGES],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.split()
    except subprocess.CalledProcessError:
        raise SystemExit(
            f"no commit {GH_PAGES}: fetch the gh-pages branch, which holds the old site"
        )
    # 404.html is the old theme's error page; nothing links to it, so it is not an address to keep.
    pages = [f for f in files if f.endswith(".html") and "/" in f and not f.endswith("/404.html")]
    # GitHub Pages serves each index.html at its bare folder too, and that is the form links use:
    # PyPI's Examples link is /latest/examples/.
    folders = [p.removesuffix("index.html") for p in pages if p.endswith("/index.html")]
    latest = [f"latest/{f.split('/', 1)[1]}" for f in pages + folders if f.startswith("2.4.2/")]
    # PyPI's Asset reference link. No page ever stood there, on the old site or the new.
    assets = ["latest/api/assets/"]
    return sorted({"/" + p for p in pages + folders + latest + assets})


def rules() -> list[tuple[re.Pattern, str]]:
    config = json.loads((ROOT / "vercel.json").read_text())
    out = []
    for rule in config["redirects"]:
        if re.search(r"/:\w", rule["source"]):
            raise SystemExit(f"named params are not checked exactly: {rule['source']}")
        out.append((re.compile("^" + rule["source"] + "$"), rule["destination"]))
    return out


def resolve(path: str, table) -> str:
    for pattern, dest in table:
        m = pattern.match(path)
        if m:
            return re.sub(r"\$(\d)", lambda g: m.group(int(g.group(1))) or "", dest)
    return path


def current_minor() -> str:
    """The highest saved folder is the current release, served at the root with latest and dev."""
    folders = json.loads((ROOT / "docs-site/archive/folders.json").read_text())
    return ".".join(max(folders, key=lambda v: tuple(map(int, v.split(".")))).split(".")[:2])


def wrong_version(old: str, new: str, current: str) -> bool:
    """An old release's address must land in that release's copy, not merely on a page.

    A copy is /vX.Y/; /vX/ and /vX.Y.Z/ are redirect stubs, and the patch ones exist only in a
    build that has tags, so they count as wrong for every address.
    """
    release = old.split("/")[1]
    minor = ".".join(release.split(".")[:2])
    if release in ("latest", "dev") or minor == current:
        return re.match(r"/v\d+(\.\d+)*/", new) is not None
    return not new.startswith(f"/v{minor}/")


def has_anchor(html: str, url: str) -> bool:
    fragment = urllib.parse.urlsplit(url).fragment
    return not fragment or re.search(rf'\sid="{re.escape(fragment)}"', html) is not None


def lands(old: str, url: str, html: Optional[str], current: str) -> bool:
    """The page an old address reaches exists, is in its version, and has the section it names."""
    path = urllib.parse.urlsplit(url).path
    return (
        html is not None
        and "/404" not in path
        and not wrong_version(old, path, current)
        and has_anchor(html, url)
    )


def landed_page(dist: Path, path: str) -> Optional[str]:
    page = path.split("#", 1)[0]
    target = dist / page.lstrip("/")
    target = target / "index.html" if page.endswith("/") else target
    return target.read_text() if target.is_file() else None


def main() -> int:
    where = sys.argv[1]
    current = current_minor()
    addresses = old_addresses()
    bad = []
    if where.startswith("http"):
        for path in addresses:
            try:
                with OPENER.open(where.rstrip("/") + path) as resp:
                    html = resp.read().decode()
                    if resp.status != 200 or not lands(path, resp.url, html, current):
                        bad.append(f"{resp.status} {path} → {resp.url}")
            except Exception as err:  # HTTPError included
                bad.append(f"{err} {path}")
    else:
        table = rules()
        for path in addresses:
            new = resolve(path, table)
            if new == path or not lands(path, new, landed_page(Path(where), new), current):
                bad.append(f"{path} → {new}")
    for line in bad:
        print(line)
    print(
        f"{len(addresses) - len(bad)} of {len(addresses)} old addresses land on their version's page"
    )
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
