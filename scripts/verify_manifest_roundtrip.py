#!/usr/bin/env python3
"""Verify that importing a manifest and re-exporting it loses no statements or blobs.

Reproduces the VCoCo/VComp manifest-import bug: `Context.import_manifest()`
stores every statement unconditionally, but `Context.export()` only pulls
statements reachable by walking outward from `ComputationRegistration`
statements (inputs/outputs/associations/registeredBy DIDs). Anything not
reachable from that walk is silently absent from the export, even though it
is sitting in the local store untouched.

Embedded JSON-LD `contexts` are intentionally ignored: `import_manifest` never
persists them, so they never survive any round trip regardless of this fix --
that's a separate, known limitation and not part of what this script checks.

Usage:
    just install-package            # build the extension against current source first
    python scripts/verify_manifest_roundtrip.py path/to/raw-manifest.json
    python scripts/verify_manifest_roundtrip.py path/to/raw-manifest.json --keep-export out.json

Exit code is 0 if the round trip is lossless, 1 otherwise.
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path
from typing import Any


def _load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _credential_subject(statement: dict[str, Any]) -> str | None:
    credential = statement.get("credential")
    if not isinstance(credential, dict):
        return None
    subject = credential.get("credentialSubject")
    if isinstance(subject, list):
        subject = subject[0] if subject else None
    if isinstance(subject, dict):
        return subject.get("id")
    return None


def _describe(statement_id: str, statement: dict[str, Any]) -> str:
    stype = statement.get("@type", "?")
    extra = ""
    if stype == "CredentialRegistration":
        subject = _credential_subject(statement)
        extra = f" credentialSubject={subject}"
    return f"{statement_id} [{stype}]{extra}"


def run_roundtrip(raw_manifest_path: Path, export_path: Path) -> dict[str, Any]:
    """Imports `raw_manifest_path` into a fresh, isolated Context and exports it
    back out to `export_path`. Returns a dict describing what was gained/lost.
    """
    import eqty_sdk

    with tempfile.TemporaryDirectory(prefix="eqty-sdk-roundtrip-") as app_dir:
        eqty_sdk.init(custom_dir=app_dir)
        ctx = eqty_sdk.Context.new("manifest-roundtrip-verification")
        ctx.import_manifest(raw_manifest_path)
        ctx.export(export_path)

    raw = _load(raw_manifest_path)
    exported = _load(export_path)

    raw_statements: dict[str, Any] = raw.get("statements", {})
    exported_statements: dict[str, Any] = exported.get("statements", {})
    raw_blobs: set[str] = set(raw.get("blobs", {}).keys())
    exported_blobs: set[str] = set(exported.get("blobs", {}).keys())

    missing_statement_ids = set(raw_statements) - set(exported_statements)
    missing_blob_ids = raw_blobs - exported_blobs

    return {
        "raw_statement_count": len(raw_statements),
        "exported_statement_count": len(exported_statements),
        "missing_statements": [
            _describe(sid, raw_statements[sid]) for sid in sorted(missing_statement_ids)
        ],
        "raw_blob_count": len(raw_blobs),
        "exported_blob_count": len(exported_blobs),
        "missing_blobs": sorted(missing_blob_ids),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path, help="Path to the raw manifest to import.")
    parser.add_argument(
        "--keep-export",
        type=Path,
        default=None,
        help="Write the re-exported manifest here instead of discarding it.",
    )
    args = parser.parse_args()

    if not args.manifest.exists():
        print(f"error: {args.manifest} does not exist", file=sys.stderr)
        return 1

    if args.keep_export is not None:
        export_path = args.keep_export
        result = run_roundtrip(args.manifest, export_path)
    else:
        with tempfile.TemporaryDirectory(prefix="eqty-sdk-roundtrip-export-") as tmp:
            export_path = Path(tmp) / "exported-manifest.json"
            result = run_roundtrip(args.manifest, export_path)

    print(
        f"statements: {result['raw_statement_count']} in -> {result['exported_statement_count']} out"
    )
    print(f"blobs:      {result['raw_blob_count']} in -> {result['exported_blob_count']} out")

    ok = True
    if result["missing_statements"]:
        ok = False
        print(f"\nMISSING {len(result['missing_statements'])} statement(s):")
        for line in result["missing_statements"]:
            print(f"  - {line}")
    if result["missing_blobs"]:
        ok = False
        print(f"\nMISSING {len(result['missing_blobs'])} blob(s):")
        for cid in result["missing_blobs"]:
            print(f"  - {cid}")
    if ok:
        print("\nOK: round trip is lossless (statements & blobs).")
        return 0

    print("\nFAILED: round trip dropped statements and/or blobs.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
