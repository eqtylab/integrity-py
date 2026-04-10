#!/usr/bin/env python3
"""Resolve the latest released SDK version from the GitHub API."""

from __future__ import annotations

import json
import os
import sys
from urllib.request import Request, urlopen

LATEST_RELEASE_API = "https://api.github.com/repos/eqtylab/integrity-py/releases/latest"
GITHUB_API_TIMEOUT_SECONDS = 10


def main() -> None:
    req = Request(LATEST_RELEASE_API, headers={"Accept": "application/vnd.github+json"})
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        req.add_header("Authorization", f"Bearer {token}")

    with urlopen(req, timeout=GITHUB_API_TIMEOUT_SECONDS) as response:
        if response.status != 200:
            raise SystemExit(f"GitHub API request failed with status {response.status}")
        data = json.loads(response.read())

    tag = data.get("tag_name", "")
    if not tag:
        raise SystemExit(f"No tag_name in GitHub API response: {data}")

    print(tag.lstrip("v"))


if __name__ == "__main__":
    sys.exit(main())
