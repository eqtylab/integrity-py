# Eqty Python SDK

Repository for developing the `eqty_sdk` source, native extension, tests, examples, and documentation.

## Usage

User-facing installation instructions, examples, and API reference live in the docs site:

- Latest release docs: <https://eqtylab.github.io/integrity-py/latest/>
- Development docs from `main`: <https://eqtylab.github.io/integrity-py/dev/>

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

1. Make sure the release commit is merged and pushed.
2. In GitHub, create a new release for the repository.
3. Enter the release tag in semver form with a `v` prefix, for example `v2.0.8`.
4. Publish the GitHub release. That creates the tag on the remote and triggers the release workflow.
5. The `Publish new release` workflow will:
   - build and publish Linux and macOS wheels
   - build and publish the source distribution
   - generate release-specific wheel requirement reports
   - publish versioned docs and update `latest`

### Versioned Docs

The docs site is versioned:

- `latest` points to the newest release
- `dev` tracks `main`
- numbered versions such as `2.0.7` map to specific releases
