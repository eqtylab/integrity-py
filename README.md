# Eqty Python SDK

Repository for developing the `eqty_sdk` source, native extension, tests, examples, and documentation.

## Usage

User-facing installation instructions, examples, and API reference live in the docs site:

- <https://integrity-py.docs.eqtylab.io/>, with older releases in its version picker

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for notable SDK changes and
[CONTRIBUTING.md](CONTRIBUTING.md) for PR and release instructions.

## Development

### Prerequisites

The easiest way to develop this repo is with the Nix flake:

```sh
nix develop
```

If you are not using Nix, install the required dependencies manually:

- [just](https://github.com/casey/just)
- [poetry](https://python-poetry.org/docs/)
- Python `3.10`
- Rust toolchain

### Environment Setup

```sh
# Configure the local Poetry environment and install dependencies
just install

# Optional: install the pre-push git hook
just init
```

The Poetry virtualenv is configured in-project at `.venv/`.

### Local Workflow

1. Make changes in `src/` for Rust or `eqty_sdk/` for Python.
2. Regenerate stubs and docs snippets when API-facing behavior changes:
   ```sh
   just generate-stubs
   ```
3. Build and install the extension into the local virtualenv:
   ```sh
   just install-package
   ```
4. Run checks before pushing:
   ```sh
   just ci
   ```

### Common Commands

Run `just` to see all available commands.

```present just --list
Available recipes:
    build                  # Build the Rust/Python wheel using maturin
    build-docs             # Builds HTML docs for the sdk
    ci                     # Run full CI pipeline: format check, lint, type check, build, and test
    fix                    # Auto-fix Rust clippy warnings
    fmt                    # Auto-format code (Rust + Python)
    fmt-check              # Check code formatting without changes (Rust + Python)
    generate-stubs         # Generate type stubs from Rust code
    init                   # Set up git hooks for prek
    install                # Install all Python dependencies via poetry
    install-package        # Install the local build of the wheel into the venv
    lint                   # Run linters and auto-fix issues (Rust clippy + Python ruff)
    lint-check             # Run linters without auto-fixing (Rust clippy + Python ruff)
    lint-docs              # Check that all public items have documentation
    readme-check           # Check if README.md is up to date with auto-generated content
    readme-update          # Update README.md with auto-generated content (Justfile commands, etc.)
    serve-docs             # Serves the documentation locally with live reload
    test-example-manifests # Run example scripts and compare normalized manifests to expected outputs
    test-py                # Run Python unit tests
    test-rs                # Run rust unit tests
    type-check             # Run mypy type checking on the Python SDK
```

### Project Structure

```text
├── eqty_sdk/              # Python package exports and pure-Python helpers
│   ├── asset/             # Asset classes
│   └── compute/           # Compute decorators and helpers
├── src/                   # Rust implementation and PyO3 bindings
│   ├── indexer/           # SQLite-backed graph and statement indexing
│   ├── integrity_service/ # Integrity service client helpers
│   └── statements/        # Statement creation and registration bindings
├── tests/                 # Python unit tests
├── integration-tests/     # Integration test assets and runners
├── examples/              # Example scripts used by docs and testing
├── docs/                  # MkDocs documentation
├── scripts/               # Development utilities
├── flake.nix              # Recommended dev environment
└── Justfile               # Common development commands
```

## Releasing

Releases are handled through GitHub and the `Publish new release` workflow in [release.yml](.github/workflows/release.yml).

### Release Steps

1. Prepare the changelog and run the release check as described in
   [CONTRIBUTING.md](CONTRIBUTING.md#releases), then merge and push the release commit.
2. In GitHub, create a new release for the repository.
3. Enter the release tag in semver form with a `v` prefix, for example `v2.0.8`.
4. Publish the GitHub release. That creates the tag on the remote and triggers the release workflow.
5. The `Publish new release` workflow will:
   - check that stable versions have a matching changelog heading
   - build and publish Linux, macOS, and Windows wheels
   - build and publish the source distribution
   - generate release-specific wheel requirement reports
   - publish versioned docs and update `latest`
   - open a `docs: archive the X.Y.Z docs` PR that saves this release's docs, with its reports
6. Check that PR's Vercel preview and merge it promptly. Until it merges, every host labels latest
   as the previous release and does not yet list it as an old version; CI fails a stale folder.
   The first release after the 2.4 series also needs `vercel.json` changed: its PR says to send the
   old `/2.4.x/` addresses to `/v2.4/`, and the docs build check fails until they go there. main
   does not require that check, so push the fix to the PR's branch before merging.

A backport to 2.0 to 2.4 runs the release workflow stored in its tag, which has no archive job,
and main's docs build fails until its docs are archived by hand. On a Mac with Docker running, in
a branch of main, with `X.Y.Z` the backport's version:

1. From the release's PyPI page, download its two `manylinux` and two `macosx` wheels into
   `wheels/`.
2. Make the Linux reports in the image release CI builds them with, then again with `aarch64` in
   place of each `x86_64`:
   ```bash
   docker run --rm -v "$PWD:/io" -w /io quay.io/pypa/manylinux2014_x86_64 sh -c \
     'py=/opt/python/cp312-cp312/bin/python && $py -m pip install -q auditwheel &&
      $py scripts/generate_auditwheel_report.py wheels/eqty_sdk-X.Y.Z-*manylinux*_x86_64.whl \
        docs/generated/auditwheel-show-linux-x86_64.txt'
   ```
3. Make the macOS reports:
   ```bash
   for arch in arm64 x86_64; do
     dir=$(mktemp -d) && unzip -q wheels/eqty_sdk-X.Y.Z-*macosx*_$arch.whl -d "$dir"
     otool -L "$dir"/eqty_sdk/*.so > docs/generated/otool-show-macos-$arch.txt
   done
   ```
4. With a Python that has `griffe==1.14.0`, `griffe2md==1.2.5` and `pyyaml`, run
   `PYTHON=<that python> scripts/archive_release.sh X.Y.Z`. It converts the tag's pages, with these
   reports, into `docs-site/archive/vX.Y/`, renames the group's key in `folders.json` and puts
   latest's reports back.
5. Commit `docs-site/archive/` and open a PR to main.

### Versioned Docs

The docs site is versioned:

- `latest` points to the newest release
- `dev` tracks `main`
- numbered versions such as `2.0.7` map to specific releases
