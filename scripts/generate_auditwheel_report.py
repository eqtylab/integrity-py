#!/usr/bin/env python3
"""Generate a normalized auditwheel show report for a wheel."""

from __future__ import annotations

import argparse
import contextlib
import io
import re
import sys
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

    wheel = args.wheel.resolve(strict=False)
    if not wheel.is_file() or wheel.suffix != ".whl":
        parser.error("wheel must be an existing .whl file")

    try:
        from auditwheel.main import main as auditwheel_main
    except ImportError:
        parser.error("the auditwheel Python package is not installed")

    output = io.StringIO()
    original_argv = sys.argv
    try:
        sys.argv = ["auditwheel", "show", str(wheel)]
        with contextlib.redirect_stdout(output):
            return_code = auditwheel_main()
    finally:
        sys.argv = original_argv

    if return_code not in (None, 0):
        raise RuntimeError(f"auditwheel show failed with exit code {return_code}")

    text = normalize_auditwheel_output(output.getvalue())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
