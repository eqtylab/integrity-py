#!/usr/bin/env python3
"""Generate a normalized auditwheel show report for a wheel."""

from __future__ import annotations

import argparse
import re
import shutil
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

    wheel = args.wheel.resolve(strict=True)
    if not wheel.is_file() or wheel.suffix != ".whl":
        parser.error("wheel must be an existing .whl file")

    auditwheel = shutil.which("auditwheel")
    if auditwheel is None:
        parser.error("auditwheel was not found on PATH")

    # The executable is resolved from PATH and the wheel is validated above. Using
    # an argument vector (and never a shell) keeps both values out of command syntax.
    output = subprocess.check_output(
        [auditwheel, "show", str(wheel)],
        text=True,
    )
    text = normalize_auditwheel_output(output)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
