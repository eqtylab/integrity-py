#!/usr/bin/env bash

set -xeu

SCRIPT_ROOT="${SCRIPT_ROOT:-/test}"
PYTHON_VERSION_FILE="${PYTHON_VERSION_FILE:-$SCRIPT_ROOT/.python-version}"
export PYTHON_VERSION_FILE

sh "$SCRIPT_ROOT/_install_python.sh"
sh "$SCRIPT_ROOT/_sys_info.sh"

EQTY_SDK_VERSION="${1:-${EQTY_SDK_VERSION:-latest}}"
TEST_SCRIPT="${2:-_basic_test.py}"
TARGET_VERSION="${EQTY_SDK_VERSION}"

INDEX_URL="https://$EQTY_PYPI_USER:$EQTY_PYPI_PASSWORD@pypi.eqtylab.io/simple/"
BREAK_SYSTEM_PACKAGES=""
if pip install --help 2>&1 | grep -q -- "--break-system-packages"; then
    BREAK_SYSTEM_PACKAGES="--break-system-packages"
fi

if [ "$EQTY_SDK_VERSION" = "latest" ]; then
    TARGET_VERSION="$(python3 "$SCRIPT_ROOT/_resolve_latest_github_version.py")"
fi

pip install \
    --index-url "$INDEX_URL" \
    --only-binary=:all: \
    ${BREAK_SYSTEM_PACKAGES:+$BREAK_SYSTEM_PACKAGES} \
    "eqty_sdk==$TARGET_VERSION"

INSTALLED_VERSION="$(python3 - <<'PY'
from importlib import metadata

print(metadata.version("eqty_sdk"))
PY
)"

if [ "$INSTALLED_VERSION" != "$TARGET_VERSION" ]; then
    echo "Installed eqty_sdk version $INSTALLED_VERSION but expected $TARGET_VERSION"
    exit 1
fi

if [ "$EQTY_SDK_VERSION" = "latest" ]; then
    echo "Installed latest eqty_sdk version: $INSTALLED_VERSION"
else
    echo "Installed requested eqty_sdk version: $INSTALLED_VERSION"
fi

python3 "$SCRIPT_ROOT/$TEST_SCRIPT"
