#!/usr/bin/env python3
"""Resolve the latest released SDK version from the GitHub API."""

from __future__ import annotations

import json
import os
import sys
from urllib.request import Request, urlopen


LATEST_RELEASE_API = "https://api.github.com/repos/eqtylab/integrity-py/releases/latest"


def main() -> None:
    req = Request(LATEST_RELEASE_API, headers={"Accept": "application/vnd.github+json"})
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        req.add_header("Authorization", f"Bearer {token}")

    with urlopen(req) as response:
        data = json.loads(response.read())

    tag = data.get("tag_name", "")
    if not tag:
        raise SystemExit(f"No tag_name in GitHub API response: {data}")

    print(tag.lstrip("v"))


if __name__ == "__main__":
    sys.exit(main())
