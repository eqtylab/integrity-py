#!/bin/bash
set -e

EQTY_SDK_VERSION="${1:-latest}"
TEST_SCRIPT="${2:-_basic_test.py}"

echo "=== eqty-sdk Integration Test ==="
echo "Testing package installation and basic functionality"
echo "Installing eqty_sdk package (version: $EQTY_SDK_VERSION)..."

if [ -n "$EQTY_PYPI_PASSWORD" ]; then
    INDEX_URL="http://eqty:$EQTY_PYPI_PASSWORD@eqty-pypi.westus2.cloudapp.azure.com/simple/"
else
    INDEX_URL="http://eqty-pypi.westus2.cloudapp.azure.com/simple/"
fi

if [ "$EQTY_SDK_VERSION" = "latest" ]; then
    pip install --index-url "$INDEX_URL" --trusted-host eqty-pypi.westus2.cloudapp.azure.com eqty_sdk
else
    pip install --index-url "$INDEX_URL" --trusted-host eqty-pypi.westus2.cloudapp.azure.com "eqty_sdk==$EQTY_SDK_VERSION"
fi

echo "Package installed successfully. Running tests..."

python3 "/test/$TEST_SCRIPT"

echo "Integration test completed successfully!"
