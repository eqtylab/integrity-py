# Default recipe: list all available commands
_:
  @just --list

_poetry-config:
  @echo "Configuring poetry"
  poetry config virtualenvs.in-project true
  poetry env use "python$(cat .python-version)"

# Install all Python dependencies via poetry
install: _poetry-config
  @echo "Installing dependencies"
  poetry install

# Set up git hooks for prek
init: install
  poetry env activate
  prek install --hook-type pre-push

# Build the Rust/Python wheel using maturin
build:
  maturin build

# Install the local build of the wheel into the venv
install-package: _stubs
  poetry run maturin develop

# Run linters without auto-fixing (Rust clippy + Python ruff)
lint-check:
  cargo clippy -- -Dwarnings --no-deps
  ruff check .

# Run linters and auto-fix issues (Rust clippy + Python ruff)
lint:
  ruff check . --fix
  cargo clippy --fix --allow-dirty --all-targets --all-features -- -Dwarnings --no-deps

# Check that all public Rust items have doc comments
lint-docs:
  cargo rustdoc --lib -- -D missing_docs -D rustdoc::broken_intra_doc_links

# Refresh the stubs and docs pages and build the site into docs-site/dist (needs `just install`, ruff, Node and pnpm)
build-docs: _render-docs
  cd ./docs-site && pnpm install && pnpm build

# Refresh the stubs and docs pages and serve the site locally with live reload (needs `just install`, ruff, Node and pnpm)
serve-docs: _render-docs
  cd ./docs-site && pnpm install && pnpm dev

# Rebuild the stubs, then fill the docs pages' generated blocks from the code and examples
_render-docs: _stub-files
  @echo "Refreshing the docs pages"
  poetry run python ./scripts/render_api_docs.py

# Auto-fix Rust clippy warnings
fix:
    cargo clippy --fix --allow-dirty -- -Dwarnings --no-deps

# Check code formatting without changes (Rust + Python)
fmt-check:
  cargo fmt -- --check
  ruff format --check

# Auto-format code (Rust + Python)
fmt:
  cargo fmt
  ruff format

# Run mypy type checking on the Python SDK
type-check:
  poetry run mypy ./eqty_sdk

# Run Python unit tests
test-py:
  poetry run python -m unittest discover tests

# Run example scripts and compare normalized manifests to expected outputs
test-example-manifests:
  poetry run python integration-tests/_example_manifests_test.py

# Run rust unit tests
test-rs:
  cargo test

# Generate type stubs from Rust code and refresh the docs pages
generate-stubs: _stubs _render-docs

# Build the type stubs from Rust code, then format and lint them. install-package runs only
# this, so a docs error never stops the build or the tests.
_stubs: _stub-files
  @echo "Formatting generated files"
  just fmt
  @echo "Linting generated files"
  just lint

# Write the type stubs and the built-in asset list from the source, without compiling the Rust,
# then sort the package stub's imports and format both stubs. Checks for ruff before writing.
_stub-files:
  @command -v ruff >/dev/null || { echo "The stubs need ruff: https://docs.astral.sh/ruff/" >&2; exit 1; }
  @echo "Generating stubs"
  poetry run python ./scripts/generate_stubs.py
  ruff check --select I --fix eqty_sdk/__init__.pyi
  ruff format eqty_sdk/_rust.pyi eqty_sdk/__init__.pyi

# Save a 2.0 to 2.4 backport's docs after it ships (macOS, Docker)
archive-backport version:
  scripts/archive_backport.sh {{version}}

# Update README.md with auto-generated content (Justfile commands, etc.)
readme-update:
  present --in-place README.md

# Check if README.md is up to date with auto-generated content
readme-check: _tmp
  present README.md > tmp/README.md
  diff README.md tmp/README.md

# Create temporary directory for artifacts
_tmp:
  mkdir -p tmp

# Run full CI pipeline: format check, lint, type check, build, test, and refresh the docs pages
ci: fmt-check readme-check lint-docs lint-check type-check install-package test-py test-example-manifests test-rs _render-docs
