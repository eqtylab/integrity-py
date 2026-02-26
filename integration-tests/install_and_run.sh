#!/usr/bin/env bash

# Integration test runner for eqty-sdk
# Builds Docker image and runs the test inside the container
# Usage: ./install-and-run.sh [VERSION]
# If VERSION is not provided, installs the latest version

set -xeu

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE="${IMAGE:-debian:bookworm}"
EQTY_SDK_VERSION="${1:-latest}"
TEST_SCRIPT="${2:-_basic_import.py}"

EQTY_PYPI_USER="${EQTY_PYPI_USER}"
EQTY_PYPI_PASSWORD="${EQTY_PYPI_PASSWORD}"

docker run --rm \
    -v "$SCRIPT_DIR:/test:ro" \
    -e "EQTY_PYPI_USER=${EQTY_PYPI_USER:-}" \
    -e "EQTY_PYPI_PASSWORD=${EQTY_PYPI_PASSWORD:-}" \
    "$IMAGE" \
    sh /test/_install_and_run.sh $EQTY_SDK_VERSION $TEST_SCRIPT

echo "Integration test completed successfully!"
