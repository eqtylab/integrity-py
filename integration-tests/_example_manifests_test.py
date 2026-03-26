#!/usr/bin/env python3
"""Run selected examples and compare normalized manifests to expected outputs."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

THIS_DIR = Path(__file__).resolve().parent
REPO_ROOT = THIS_DIR.parent
EXPECTED_DIR = THIS_DIR / "examples"
VOLATILE_STATEMENT_FIELDS = {"@context", "@id", "registeredBy", "operatedBy", "timestamp"}


@dataclass(frozen=True)
class ExampleCase:
    name: str
    script: Path
    manifests: tuple[str, ...]


EXAMPLES = (
    ExampleCase(
        "basic-workflow",
        REPO_ROOT / "examples/basic-workflow.py",
        ("manifests/basic-workflow.json",),
    ),
    ExampleCase(
        "creating-the-model",
        REPO_ROOT / "examples/creating-the-model.py",
        ("manifests/default-ctx.json",),
    ),
    ExampleCase(
        "context-linking",
        REPO_ROOT / "examples/context-linking.py",
        ("manifests/customer-project.json", "manifests/daily-run-2026-03-25.json"),
    ),
    ExampleCase(
        "path-backed-assets",
        REPO_ROOT / "examples/path-backed-assets.py",
        ("manifests/path-backed-assets.json",),
    ),
    ExampleCase(
        "using-the-model",
        REPO_ROOT / "examples/using-the-model.py",
        ("manifests/run-ctx.json",),
    ),
    ExampleCase(
        "model-signing",
        REPO_ROOT / "examples/model-signing.py",
        ("manifests/model-signing.json",),
    ),
)


def _replace_statement_refs(value: Any, statement_ids: set[str]) -> Any:
    if isinstance(value, str):
        return "<statement>" if value in statement_ids else value
    if isinstance(value, list):
        return [_replace_statement_refs(item, statement_ids) for item in value]
    if isinstance(value, dict):
        return {key: _replace_statement_refs(item, statement_ids) for key, item in value.items()}
    return value


def _normalize_statement(statement: dict[str, Any], statement_ids: set[str]) -> dict[str, Any]:
    normalized = {
        key: _replace_statement_refs(value, statement_ids)
        for key, value in statement.items()
        if key not in VOLATILE_STATEMENT_FIELDS
    }
    if normalized.get("@type") == "CredentialRegistration" and "sigstoreBundle" in normalized:
        normalized["sigstoreBundle"] = "<sigstore bundle>"
    return normalized


def normalize_manifest(manifest: dict[str, Any]) -> dict[str, Any]:
    statement_ids = set(manifest.get("statements", {}).keys())
    return {
        "version": manifest.get("version"),
        "statements": sorted(
            (
                _normalize_statement(statement, statement_ids)
                for statement in manifest.get("statements", {}).values()
            ),
            key=lambda item: json.dumps(item, sort_keys=True),
        ),
        "blobs": manifest.get("blobs", {}),
    }


def _run_example(example: ExampleCase, workdir: Path) -> None:
    env = os.environ.copy()
    env.pop("PYTHONPATH", None)
    (workdir / "manifests").mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [sys.executable, str(example.script)],
        cwd=workdir,
        env=env,
        check=True,
    )


def _load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _expected_path(example: ExampleCase, manifest_name: str) -> Path:
    if len(example.manifests) == 1:
        return EXPECTED_DIR / f"{example.name}.json"
    stem = Path(manifest_name).stem
    return EXPECTED_DIR / f"{example.name}.{stem}.json"


def _assert_or_update(
    example: ExampleCase, manifest_name: str, actual: dict[str, Any], update: bool
) -> None:
    expected_path = _expected_path(example, manifest_name)
    if update:
        expected_path.parent.mkdir(parents=True, exist_ok=True)
        expected_path.write_text(
            json.dumps(actual, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        return

    expected = _load_json(expected_path)
    if actual != expected:
        raise AssertionError(
            f"{example.name}:{manifest_name} manifest mismatch.\n"
            f"Expected: {expected_path}\n"
            "Run `_example_manifests_test.py --update-expected` to refresh after intentional changes."
        )


def run_examples(update: bool) -> None:
    for example in EXAMPLES:
        with tempfile.TemporaryDirectory(prefix=f"eqty-example-{example.name}-") as tmpdir:
            workdir = Path(tmpdir)
            _run_example(example, workdir)
            for manifest_name in example.manifests:
                manifest_path = workdir / manifest_name
                actual = normalize_manifest(_load_json(manifest_path))
                _assert_or_update(example, manifest_name, actual, update)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--update-expected", action="store_true")
    args = parser.parse_args()

    if args.update_expected and shutil.which("git") is None:
        raise RuntimeError("git must be available when updating expected manifests")

    run_examples(update=args.update_expected)
    print("Example manifest checks passed")


if __name__ == "__main__":
    main()
