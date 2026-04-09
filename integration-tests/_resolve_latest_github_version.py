#!/usr/bin/env python3
"""Resolve the latest released SDK version from GitHub releases."""

from __future__ import annotations

import re
import sys
from urllib.request import urlopen


LATEST_RELEASE_URL = "https://github.com/eqtylab/integrity-py/releases/latest"


def main() -> None:
    with urlopen(LATEST_RELEASE_URL) as response:
        final_url = response.geturl()

    match = re.search(r"/tag/v(\d+(?:\.\d+)*)/?$", final_url)
    if match is None:
        raise SystemExit(f"Could not determine latest release from {final_url}")

    print(match.group(1))


if __name__ == "__main__":
    sys.exit(main())
