#!/usr/bin/env bash

# Integration test runner for eqty-sdk
# Builds Docker image and runs the test inside the container
# Usage: ./install-and-run.sh [VERSION]
# If VERSION is not provided, installs latest

set -xeu

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
IMAGE="${IMAGE:-debian:bookworm}"
EQTY_SDK_VERSION="${1:-${EQTY_SDK_VERSION:-latest}}"
TEST_SCRIPT="${2:-_basic_import.py}"

EQTY_PYPI_USER="${EQTY_PYPI_USER}"
EQTY_PYPI_PASSWORD="${EQTY_PYPI_PASSWORD}"

docker run --rm \
    -v "$SCRIPT_DIR:/test:ro" \
    -v "$REPO_ROOT/.python-version:/test/.python-version:ro" \
    -e "EQTY_PYPI_USER=${EQTY_PYPI_USER:-}" \
    -e "EQTY_PYPI_PASSWORD=${EQTY_PYPI_PASSWORD:-}" \
    -e "EQTY_SDK_VERSION=${EQTY_SDK_VERSION:-}" \
    "$IMAGE" \
    sh /test/_install_and_run.sh "$EQTY_SDK_VERSION" "$TEST_SCRIPT"

echo "Integration test completed successfully!"
