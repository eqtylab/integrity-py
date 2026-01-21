#!/bin/bash
set -e

# Integration test runner for eqty-sdk
# Builds Docker image and runs the test inside the container
# Usage: ./install-and-run.sh [VERSION]
# If VERSION is not provided, installs the latest version

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE="python:3.11-slim"
EQTY_SDK_VERSION="${1:-latest}"
TEST_SCRIPT="${2:-_basic_test.py}"

echo "Running integration test (script: $TEST_SCRIPT) in Docker container (version: $VERSION)..."
docker run --rm \
    -v "$SCRIPT_DIR/_install-and-run.sh:/test/_install-and-run.sh:ro" \
    -v "$SCRIPT_DIR/$TEST_SCRIPT:/test/$TEST_SCRIPT:ro" \
    -e "EQTY_PYPI_PASSWORD=${EQTY_PYPI_PASSWORD:-}" \
    "$IMAGE" \
    bash /test/_install-and-run.sh $EQTY_SDK_VERSION $TEST_SCRIPT

echo "Integration test completed successfully!"
