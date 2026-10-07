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
- [maturin](https://www.maturin.rs/), to build the extension
- [ruff](https://docs.astral.sh/ruff/), which `just generate-stubs` and `just lint` run
- [present](https://github.com/terror/present), which `just ci` runs to check this README
- [Node.js](https://nodejs.org/) 22 and [pnpm](https://pnpm.io/), for the docs site

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
2. Regenerate the type stubs and the docs pages when API-facing behavior changes, then commit
   the pages it changed. The text between a page's `{/* generated ... */}` and
   `{/* end generated */}` lines is rewritten each time, so change the code, not that text:
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
    archive-backport version # Save a 2.0 to 2.4 backport's docs after it ships (macOS, Docker)
    build                    # Build the Rust/Python wheel using maturin
    build-docs               # Refresh the docs pages and build the site into docs-site/dist (needs Node and pnpm)
    ci                       # Run full CI pipeline: format check, lint, type check, build, and test
    fix                      # Auto-fix Rust clippy warnings
    fmt                      # Auto-format code (Rust + Python)
    fmt-check                # Check code formatting without changes (Rust + Python)
    generate-stubs           # Generate type stubs from Rust code and refresh the docs pages
    init                     # Set up git hooks for prek
    install                  # Install all Python dependencies via poetry
    install-package          # Install the local build of the wheel into the venv
    lint                     # Run linters and auto-fix issues (Rust clippy + Python ruff)
    lint-check               # Run linters without auto-fixing (Rust clippy + Python ruff)
    lint-docs                # Check that all public items have documentation
    readme-check             # Check if README.md is up to date with auto-generated content
    readme-update            # Update README.md with auto-generated content (Justfile commands, etc.)
    serve-docs               # Refresh the docs pages and serve the site locally with live reload (needs Node and pnpm)
    test-example-manifests   # Run example scripts and compare normalized manifests to expected outputs
    test-py                  # Run Python unit tests
    test-rs                  # Run rust unit tests
    type-check               # Run mypy type checking on the Python SDK
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
├── docs-site/             # Documentation site
├── docs/generated/        # Generated reports and lists, copied into the docs pages
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
   - open a `docs: archive the X.Y.Z docs` PR that saves this release's docs, with its reports
6. Check that PR's Vercel preview and merge it promptly. Until it merges, the site labels latest
   as the previous release and does not yet list it as an old version; CI fails a stale folder.
   The first release after the 2.4 series also needs `vercel.json` changed: its PR says to send the
   old `/2.4.x/` addresses to `/v2.4/`, and the docs build check fails until they go there. main
   does not require that check, so push the fix to the PR's branch before merging.

#### Backports to 2.0 to 2.4

A backport, such as 2.4.3 after 2.5.0, runs the release workflow stored in its own tag. Tags from
2.0 to 2.4 predate the step that saves each release's docs, so main's docs build fails until
someone saves them by hand. After the release has published, on an up-to-date main, on a Mac
with Docker running:

```bash
just archive-backport 2.4.3
```

It makes the wheel reports from the published wheels, as release CI would have, and rebuilds
`docs-site/archive/v2.4/` from the tag with them. If something is missing, it says what. When it
finishes, it prints the commands that put the change in a PR. It takes a few minutes, most of
them the first download of two Docker images.

It reads the wheels from the index the tag's workflow uploaded them to. Tags up to 2.3.0 upload
only to EQTY Lab's index, `pypi.eqtylab.io`, so a backport cut from one of them does too; later
tags upload to PyPI. EQTY Lab's index needs a login, which curl reads from `~/.netrc`. Add it
with an editor, not `echo`, so the password stays out of your shell history:

```text
machine pypi.eqtylab.io login YOUR_NAME password YOUR_PASSWORD
```

### Versioned Docs

The docs site is `docs-site/`, which Vercel builds from `main` and serves at
<https://integrity-py.docs.eqtylab.io/>. Run it locally with `just serve-docs`, which needs Node and pnpm.

- `/` is the newest release.
- Older minor releases are saved folders under `docs-site/archive/`, listed in
  `docs-site/archive/folders.json` and served at `/v2.3/`, `/v2.2/` and so on. A patch address
  such as `/v2.1.2/` redirects to its minor.
- Old `eqtylab.github.io/integrity-py/` links still work. GitHub Pages serves only
  `docs-site/forwarder/`, which sends each path to the same path on the new domain, and
  [vercel.json](vercel.json) redirects the old addresses there to their pages.
- Keep the `gh-pages` branch, though Pages no longer publishes it. Its commit `cb7be3c` holds the
  old site, which `scripts/check_old_links.py` and `scripts/archive_version.py` read.
