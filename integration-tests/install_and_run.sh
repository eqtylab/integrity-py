#!/usr/bin/env bash

# Integration test runner for eqty-sdk
# Builds Docker image and runs the test inside the container
# Usage: ./install-and-run.sh [VERSION]
# If VERSION is not provided, installs latest

set -xeu

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE="${IMAGE:-python:3.12-slim-bookworm}"
EQTY_SDK_VERSION="${1:-${EQTY_SDK_VERSION:-latest}}"
TEST_SCRIPT="${2:-_basic_import.py}"

# Resolve "latest" here on the runner where GITHUB_TOKEN is available,
# so the container always receives an explicit version number.
if [ "$EQTY_SDK_VERSION" = "latest" ]; then
    EQTY_SDK_VERSION="$(python3 "$SCRIPT_DIR/_resolve_latest_github_version.py")" || exit 1
fi

docker run --rm \
    -v "$SCRIPT_DIR:/test:ro" \
    -e "EQTY_SDK_VERSION=${EQTY_SDK_VERSION:-}" \
    "$IMAGE" \
    sh /test/_install_and_run.sh "$EQTY_SDK_VERSION" "$TEST_SCRIPT"

echo "Integration test completed successfully!"
