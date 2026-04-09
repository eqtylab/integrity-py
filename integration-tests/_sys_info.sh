#!/usr/bin/env bash

set -eu

if command -v ldd >/dev/null 2>&1; then
    RUNTIME_INFO="$(ldd --version | head -n 1)"
elif command -v sw_vers >/dev/null 2>&1; then
    RUNTIME_INFO="macOS $(sw_vers -productVersion)"
else
    RUNTIME_INFO="unknown"
fi

echo ""
echo "==== Environment Information ===="
echo "Runtime: ${RUNTIME_INFO}"
echo "Python Version: $(python3 --version)"
echo "Pip Version: $(pip --version)"
echo "================================"
echo ""
