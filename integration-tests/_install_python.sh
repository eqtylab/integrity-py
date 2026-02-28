#!/usr/bin/env bash

# Debian (+ Ubuntu, etc)
if [ -f /etc/os-release ] && grep -q "debian" /etc/os-release; then
    apt-get update && apt-get install -y python3-pip
fi

# Alpine
if [ -f /etc/os-release ] && grep -q "alpine" /etc/os-release; then
    apk add --no-cache py3-pip
fi

