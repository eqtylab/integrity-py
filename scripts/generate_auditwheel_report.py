#!/usr/bin/env python3
"""Generate a normalized auditwheel show report for a wheel."""

from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path


def normalize_auditwheel_output(text: str) -> str:
    text = text.strip() + "\n"

    def normalize_set(match: re.Match[str]) -> str:
        items = sorted(re.findall(r"'([^']+)'", match.group(0)))
        return "{" + ", ".join(f"'{item}'" for item in items) + "}"

    return re.sub(r"\{[^{}]*\}", normalize_set, text, flags=re.DOTALL)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("wheel", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    result = subprocess.run(
        ["auditwheel", "show", str(args.wheel)],
        check=True,
        capture_output=True,
        text=True,
    )
    text = normalize_auditwheel_output(result.stdout)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
