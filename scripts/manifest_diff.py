#!/usr/bin/env python3
"""Print a unified diff between two manifests after canonical JSON serialization."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

VOLATILE_STATEMENT_FIELDS = {"@context", "@id", "registeredBy", "operatedBy", "timestamp"}
VOLATILE_CREDENTIAL_FIELDS = {"id", "issuanceDate", "validFrom"}
VOLATILE_PROOF_FIELDS = {"created", "jws"}


@dataclass
class CompareOptions:
    ignore_contexts: bool = False
    ignore_timestamps: bool = False
    ignore_credentials: bool = False
    ignore_statement_types: list[str] = field(default_factory=list)
    ignore_fields: list[str] = field(default_factory=list)
    profile: str | None = None


PROFILE_DEFAULTS: dict[str, dict[str, Any]] = {
    "strict": {},
    "ci": {
        "ignore_contexts": True,
        "ignore_timestamps": True,
        "ignore_credentials": True,
    },
    "lenient": {
        "ignore_contexts": True,
        "ignore_timestamps": True,
        "ignore_credentials": True,
    },
}


def _load_manifest(value: str) -> tuple[Any, str]:
    path = Path(value)
    if path.exists():
        return json.loads(path.read_text(encoding="utf-8")), str(path)
    return json.loads(value), "<inline-json>"


def _canonical_json(value: Any) -> str:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False
    )


def _pretty_canonical_json(value: Any) -> str:
    canonical = json.loads(_canonical_json(value))
    return (
        json.dumps(canonical, ensure_ascii=False, sort_keys=True, indent=2, allow_nan=False) + "\n"
    )


def _parse_args() -> tuple[str, str, CompareOptions]:
    parser = argparse.ArgumentParser(
        description="Diff two manifests provided either as file paths or inline JSON strings."
    )
    parser.add_argument("left", help="Path to a manifest JSON file or an inline JSON string.")
    parser.add_argument("right", help="Path to a manifest JSON file or an inline JSON string.")
    parser.add_argument(
        "--ignore-contexts",
        action="store_true",
        help="Ignore top-level contexts and statement-level @context fields.",
    )
    parser.add_argument(
        "--ignore-timestamps",
        action="store_true",
        help="Ignore timestamp-like fields in statements and credentials.",
    )
    parser.add_argument(
        "--ignore-credentials",
        action="store_true",
        help="Ignore CredentialRegistration statements entirely.",
    )
    parser.add_argument(
        "--ignore-field",
        action="append",
        default=[],
        metavar="PATH",
        help="Ignore a dotted field path anywhere in the manifest, e.g. contexts or credential.proof.jws.",
    )
    parser.add_argument(
        "--ignore-statement-type",
        action="append",
        default=[],
        metavar="TYPE",
        help="Ignore statements of a given @type, e.g. CredentialRegistration.",
    )
    parser.add_argument(
        "--profile",
        choices=sorted(PROFILE_DEFAULTS),
        help="Apply a preset bundle of comparison options.",
    )
    args = parser.parse_args()

    options = CompareOptions(profile=args.profile)
    if args.profile:
        for key, value in PROFILE_DEFAULTS[args.profile].items():
            setattr(options, key, value)

    options.ignore_contexts = options.ignore_contexts or args.ignore_contexts
    options.ignore_timestamps = options.ignore_timestamps or args.ignore_timestamps
    options.ignore_credentials = options.ignore_credentials or args.ignore_credentials
    options.ignore_statement_types.extend(args.ignore_statement_type)
    if options.ignore_credentials:
        options.ignore_statement_types.append("CredentialRegistration")
    options.ignore_fields.extend(args.ignore_field)

    return args.left, args.right, options


def _replace_statement_refs(value: Any, statement_ids: set[str]) -> Any:
    if isinstance(value, str):
        return "<statement>" if value in statement_ids else value
    if isinstance(value, list):
        return [_replace_statement_refs(item, statement_ids) for item in value]
    if isinstance(value, dict):
        return {key: _replace_statement_refs(item, statement_ids) for key, item in value.items()}
    return value


def _normalize_unordered_fields(statement: dict[str, Any]) -> dict[str, Any]:
    normalized = dict(statement)
    for field_name in ("association", "input"):
        value = normalized.get(field_name)
        if isinstance(value, list) and all(isinstance(item, str) for item in value):
            normalized[field_name] = sorted(value)
    return normalized


def _normalize_credential(
    value: dict[str, Any], statement_ids: set[str], options: CompareOptions
) -> dict[str, Any]:
    normalized = _replace_statement_refs(value, statement_ids)
    if not isinstance(normalized, dict):
        return {"value": normalized}

    if options.ignore_contexts:
        normalized.pop("@context", None)

    if options.ignore_timestamps:
        for field_name in VOLATILE_CREDENTIAL_FIELDS:
            normalized.pop(field_name, None)
        proof = normalized.get("proof")
        if isinstance(proof, dict):
            for field_name in VOLATILE_PROOF_FIELDS:
                proof.pop(field_name, None)

    return normalized


def _normalize_statement(
    statement: dict[str, Any], statement_ids: set[str], options: CompareOptions
) -> dict[str, Any] | None:
    if statement.get("@type") in set(options.ignore_statement_types):
        return None

    normalized = {
        key: _replace_statement_refs(value, statement_ids)
        for key, value in statement.items()
        if key not in VOLATILE_STATEMENT_FIELDS
    }

    if options.ignore_contexts:
        normalized.pop("@context", None)

    if options.ignore_timestamps:
        normalized.pop("timestamp", None)

    if normalized.get("@type") == "CredentialRegistration":
        credential = normalized.get("credential")
        if isinstance(credential, dict):
            normalized["credential"] = _normalize_credential(credential, statement_ids, options)
        if "sigstoreBundle" in normalized:
            normalized["sigstoreBundle"] = "<sigstore bundle>"

    return _normalize_unordered_fields(normalized)


def _remove_path(value: Any, parts: list[str]) -> Any:
    if not parts:
        return value

    part = parts[0]
    rest = parts[1:]

    if isinstance(value, dict):
        result: dict[str, Any] = {}
        for key, item in value.items():
            if key == part:
                if rest:
                    result[key] = _remove_path(item, rest)
                continue
            result[key] = _remove_path(item, parts)
        return result

    if isinstance(value, list):
        return [_remove_path(item, parts) for item in value]

    return value


def _apply_ignored_fields(manifest: dict[str, Any], options: CompareOptions) -> dict[str, Any]:
    normalized = manifest
    for path in options.ignore_fields:
        parts = [part for part in path.split(".") if part]
        if parts:
            normalized = _remove_path(normalized, parts)
    return normalized


def normalize_manifest(manifest: dict[str, Any], options: CompareOptions) -> dict[str, Any]:
    statement_ids = set(manifest.get("statements", {}).keys())
    normalized = {
        "version": manifest.get("version"),
        "statements": sorted(
            (
                normalized_statement
                for statement in manifest.get("statements", {}).values()
                if (normalized_statement := _normalize_statement(statement, statement_ids, options))
                is not None
            ),
            key=lambda item: json.dumps(item, sort_keys=True),
        ),
        "blobs": manifest.get("blobs", {}),
    }
    if not options.ignore_contexts and "contexts" in manifest:
        normalized["contexts"] = manifest.get("contexts", {})
    return _apply_ignored_fields(normalized, options)


def _write_normalized_manifest(path: Path, manifest: dict[str, Any]) -> None:
    path.write_text(_pretty_canonical_json(manifest), encoding="utf-8")


def main() -> int:
    left_arg, right_arg, options = _parse_args()
    left_manifest, left_label = _load_manifest(left_arg)
    right_manifest, right_label = _load_manifest(right_arg)

    normalized_left = normalize_manifest(left_manifest, options)
    normalized_right = normalize_manifest(right_manifest, options)
    jd = shutil.which("jd")
    if jd is None:
        raise RuntimeError(
            "`jd` was not found on PATH. Install it or enter the nix dev shell first."
        )

    with tempfile.TemporaryDirectory(prefix="manifest-diff-") as tmpdir:
        tmpdir_path = Path(tmpdir)
        left_tmp = tmpdir_path / "left.normalized.json"
        right_tmp = tmpdir_path / "right.normalized.json"
        _write_normalized_manifest(left_tmp, normalized_left)
        _write_normalized_manifest(right_tmp, normalized_right)

        cmd = [jd]
        if sys.stdout.isatty():
            cmd.append("-color")
        cmd.extend([str(left_tmp), str(right_tmp)])
        result = subprocess.run(cmd, check=False)
        return result.returncode


if __name__ == "__main__":
    sys.exit(main())
