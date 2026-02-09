#!/usr/bin/env python3
"""Merge pyo3-stubgen output with extra manual stubs."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STUB_PATH = ROOT / "eqty_sdk" / "_rust.pyi"
EXTRA_PATH = ROOT / "eqty_sdk" / "_rust_extra.pyi"

MARKER = "# -- extra stubs --\n"


def main() -> int:
    if not STUB_PATH.exists():
        print(f"Missing base stub: {STUB_PATH}")
        return 1
    if not EXTRA_PATH.exists():
        print(f"Missing extra stub: {EXTRA_PATH}")
        return 1

    base = STUB_PATH.read_text()
    extra = EXTRA_PATH.read_text()

    if MARKER in base:
        base = base.split(MARKER)[0].rstrip() + "\n"

    merged = base.rstrip() + "\n\n" + MARKER + extra.lstrip()
    STUB_PATH.write_text(merged)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
