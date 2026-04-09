#!/usr/bin/env bash

set -eu

MIN_PYTHON_VERSION="$(tr -d '[:space:]' < /test/.python-version)"
MIN_PYTHON_MAJOR="${MIN_PYTHON_VERSION%%.*}"
MIN_PYTHON_MINOR="${MIN_PYTHON_VERSION#${MIN_PYTHON_MAJOR}.}"

ensure_python_gte_min() {
    python3 - <<'PY'
import sys
from pathlib import Path

minimum = Path("/test/.python-version").read_text(encoding="utf-8").strip()
major_str, minor_str = minimum.split(".", 1)
minimum_tuple = (int(major_str), int(minor_str))

if sys.version_info < minimum_tuple:
    raise SystemExit(
        f"Python {minimum}+ is required for eqty_sdk, found {sys.version.split()[0]}"
    )
PY
}

# Debian (+ Ubuntu, etc)
if [ -f /etc/os-release ] && grep -q "debian" /etc/os-release; then
    apt-get update && apt-get install -y python3-pip
fi

# Alpine
if [ -f /etc/os-release ] && grep -q "alpine" /etc/os-release; then
    apk add --no-cache py3-pip
fi

# Fedora / RHEL-like
if [ -f /etc/os-release ] && grep -Eq "fedora|rhel|ubi" /etc/os-release; then
    dnf install -y python pip
fi

# UBI 9 / RHEL 9 default to Python 3.9. Install the oldest packaged Python
# that satisfies the configured minimum version.
if [ -f /etc/os-release ] && grep -qE '^VERSION_ID="9"' /etc/os-release && grep -qE "rhel|ubi" /etc/os-release; then
    for candidate in 3.10 3.11 3.12 3.13; do
        candidate_major="${candidate%%.*}"
        candidate_minor="${candidate#${candidate_major}.}"
        if [ "$candidate_major" -lt "$MIN_PYTHON_MAJOR" ] || {
            [ "$candidate_major" -eq "$MIN_PYTHON_MAJOR" ] && [ "$candidate_minor" -lt "$MIN_PYTHON_MINOR" ]
        }; then
            continue
        fi

        if dnf install -y "python${candidate}" "python${candidate}-pip"; then
            ln -sf "/usr/bin/python${candidate}" /usr/local/bin/python3
            ln -sf "/usr/bin/pip${candidate}" /usr/local/bin/pip
            break
        fi
    done
fi

ensure_python_gte_min
