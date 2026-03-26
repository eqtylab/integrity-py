# Minimum Supported Versions

- Python: 3.10
- pip: latest supported release
- Linux wheel runtime requirements are release- and architecture-specific. See the reports below.

Pre-compiled wheels are available for common systems. If a wheel is not available for your target platform, use [Install From Source](source.md) instead.

## Linux Wheel Requirements

These reports are generated during the release workflow from the actual published Linux wheels and contain the output of `auditwheel show`.

### x86_64

```text
--8<-- "docs/generated/auditwheel-show-linux-x86_64.txt"
```

### aarch64

```text
--8<-- "docs/generated/auditwheel-show-linux-aarch64.txt"
```

## macOS Wheel Requirements

This report is generated during the release workflow from the actual published macOS wheel and contains the output of `otool -L`.
```text
--8<-- "docs/generated/otool-show-macos.txt"
```
