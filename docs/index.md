# eqty_sdk Documentation

API reference for the Python SDK.

Versioned docs are published from the release workflow:

- `dev` tracks the `main` branch
- `latest` tracks the newest release
- numbered versions such as `1.2.3` map to specific released SDK versions

## Minimum Supported Versions

- Python: 3.10
- pip: latest supported release
- glibc for Linux wheels: 2.17+ (manylinux2014)

## Linux Wheel Requirements

These reports are generated during the release workflow from the actual published Linux wheels.

### x86_64

```text
--8<-- "docs/generated/auditwheel-show-linux-x86_64.txt"
```

### aarch64

```text
--8<-- "docs/generated/auditwheel-show-linux-aarch64.txt"
```

## macOS Wheel Requirements

This report is generated during the release workflow from the actual published macOS wheel.

```text
--8<-- "docs/generated/otool-show-macos.txt"
```
