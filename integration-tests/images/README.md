# Import Test Images

Custom images used by the import test CI matrix. Each image is built from a base OS image with Python and pip pre-installed, then published to `ghcr.io/eqtylab`.

Images are rebuilt automatically when their Dockerfile changes (via `build-import-test-images.yml`), or can be triggered manually via `workflow_dispatch`.

## Image Matrix

| Directory | ghcr.io Image | Base OS | glibc | Python | pip | Expected Wheel |
|---|---|---|---|---|---|---|
| `centos7-python310` | `import-test-centos7-python310` | CentOS 7 | 2.17 | 3.10 | bundled | `manylinux_2_17` |
| `ubuntu2204-python310` | `import-test-ubuntu2204-python310` | Ubuntu 22.04 | 2.35 | 3.10 | apt | `manylinux_2_17` |
| `ubuntu2404-python312` | `import-test-ubuntu2404-python312` | Ubuntu 24.04 | 2.39 | 3.12 | apt | `manylinux_2_17` |
| `debian-bookworm-python311` | `import-test-debian-bookworm-python311` | Debian Bookworm | 2.36 | 3.11 | apt | `manylinux_2_17` |
| `ubi9-python311` | `import-test-ubi9-python311` | RHEL UBI 9 | 2.34 | 3.11 | dnf | `manylinux_2_17` |
| `ubi10-python312` | `import-test-ubi10-python312` | RHEL UBI 10 | 2.36 | 3.12 | dnf | `manylinux_2_17` |
| `alpine322-python312` | `import-test-alpine322-python312` | Alpine 3.22 | musl | 3.12 | apk | `musllinux_1_2` |

All images are tagged `:latest` on ghcr.io.

## Updating an Image

To upgrade Python, pip, or the base OS version:
1. Edit the relevant `Dockerfile`
2. Rename the directory and update references in:
   - `.github/workflows/build-import-test-images.yml`
   - `.github/workflows/import-tests.yml`
   - This README
3. Merge to `main` — the build workflow will push the new image automatically
