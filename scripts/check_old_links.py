"""Every address the old MkDocs site served must end on a page of the new site.

    python3 scripts/check_old_links.py docs-site/dist            # rules against a local build
    python3 scripts/check_old_links.py https://<preview-url>     # real requests

The list is origin/gh-pages as it stands (commit cb7be3c), less the theme's 404.html pages, so
it covers every link anyone can hold. Locally, vercel.json's redirect rules are applied
as regexes; they are written with literal text and regex groups only so that this is exact.
"""

import json
import re
import subprocess
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GH_PAGES = "cb7be3c"


class _Follow308(urllib.request.HTTPRedirectHandler):
    """Vercel's permanent redirects are 308s, which urllib follows only from Python 3.11."""

    http_error_308 = urllib.request.HTTPRedirectHandler.http_error_302

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return super().redirect_request(req, fp, 307 if code == 308 else code, msg, headers, newurl)


OPENER = urllib.request.build_opener(_Follow308())


def old_addresses() -> list[str]:
    files = subprocess.run(
        ["git", "ls-tree", "-r", "--name-only", GH_PAGES],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split()
    # 404.html is the old theme's error page; nothing links to it, so it is not an address to keep.
    pages = [f for f in files if f.endswith(".html") and "/" in f and not f.endswith("/404.html")]
    latest = [f"latest/{f.split('/', 1)[1]}" for f in pages if f.startswith("2.4.2/")]
    return sorted({"/" + p for p in pages + latest})


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


def exists(dist: Path, path: str) -> bool:
    page = path.split("#", 1)[0]
    target = dist / page.lstrip("/")
    return (target / "index.html").is_file() if page.endswith("/") else target.is_file()


def main() -> int:
    where = sys.argv[1]
    bad = []
    if where.startswith("http"):
        for path in old_addresses():
            try:
                with OPENER.open(where.rstrip("/") + path) as resp:
                    if resp.status != 200 or "/404" in resp.url:
                        bad.append(f"{resp.status} {path} → {resp.url}")
            except Exception as err:  # HTTPError included
                bad.append(f"{err} {path}")
    else:
        table = rules()
        for path in old_addresses():
            new = resolve(path, table)
            if new == path or not exists(Path(where), new):
                bad.append(f"{path} → {new}")
    for line in bad:
        print(line)
    print(
        f"{len(old_addresses()) - len(bad)} of {len(old_addresses())} old addresses land on a page"
    )
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
