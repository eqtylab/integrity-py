#!/usr/bin/env bash

set -xeu

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EQTY_SDK_VERSION="${1:-${EQTY_SDK_VERSION:-latest}}"
TEST_SCRIPT="${2:-_basic_import.py}"

EQTY_PYPI_USER="${EQTY_PYPI_USER}"
EQTY_PYPI_PASSWORD="${EQTY_PYPI_PASSWORD}"

# Resolve "latest" here on the runner where GITHUB_TOKEN is available,
# so _install_and_run.sh always receives an explicit version number.
if [ "$EQTY_SDK_VERSION" = "latest" ]; then
    EQTY_SDK_VERSION="$(python3 "$SCRIPT_DIR/_resolve_latest_github_version.py")" || exit 1
fi

export EQTY_PYPI_USER EQTY_PYPI_PASSWORD EQTY_SDK_VERSION
export SCRIPT_ROOT="$SCRIPT_DIR"
export PYTHON_VERSION_FILE="$REPO_ROOT/.python-version"

sh "$SCRIPT_DIR/_install_and_run.sh" "$EQTY_SDK_VERSION" "$TEST_SCRIPT"
