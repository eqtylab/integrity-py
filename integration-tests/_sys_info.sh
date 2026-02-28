#!/usr/bin/env bash

echo ""
echo "==== Environment Information ===="
echo "GLIBC Version: $(ldd --version | head -n 1)"
echo "Python Version: $(python3 --version)"
echo "Pip Version: $(pip --version)"
echo "================================"
echo ""
