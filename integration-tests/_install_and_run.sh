#!/usr/bin/env bash

set -xeu

sh /test/_install_python.sh
sh /test/_sys_info.sh

EQTY_SDK_VERSION="${1:-latest}"
TEST_SCRIPT="${2:-_basic_test.py}"

INDEX_URL="http://$EQTY_PYPI_USER:$EQTY_PYPI_PASSWORD@eqty-pypi.westus2.cloudapp.azure.com/simple/"
BREAK_SYSTEM_PACKAGES=()
if pip install --help 2>&1 | grep -q -- "--break-system-packages"; then
    BREAK_SYSTEM_PACKAGES=(--break-system-packages)
fi

if [ "$EQTY_SDK_VERSION" = "latest" ]; then
    pip install \
        --index-url "$INDEX_URL" \
        --trusted-host eqty-pypi.westus2.cloudapp.azure.com \
        "${BREAK_SYSTEM_PACKAGES[@]}" \
        eqty_sdk
else
    pip install \
        --index-url "$INDEX_URL" \
        --trusted-host eqty-pypi.westus2.cloudapp.azure.com \
        "${BREAK_SYSTEM_PACKAGES[@]}" \
        "eqty_sdk==$EQTY_SDK_VERSION"
fi

python3 "/test/$TEST_SCRIPT"
