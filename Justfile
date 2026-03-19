# Default recipe: list all available commands
_:
  @just --list

_poetry-config:
  @echo "Configuring poetry"
  poetry config virtualenvs.in-project true

# Install all Python dependencies via poetry
install: _poetry-config
  @echo "Installing dependencies"
  poetry install --with docs

# Set up git hooks for prek
init: install
  poetry env activate
  prek install --hook-type pre-push

# Build the Rust/Python wheel using maturin
build:
  maturin build

# Install the local build of the wheel into the venv
install-package: generate-stubs
  poetry run maturin develop

# Run linters without auto-fixing (Rust clippy + Python ruff)
lint-check:
  cargo clippy -- -Dwarnings --no-deps
  ruff check .

# Run linters and auto-fix issues (Rust clippy + Python ruff)
lint:
  ruff check . --fix
  cargo clippy --fix --allow-dirty --all-targets --all-features -- -Dwarnings --no-deps

# Check that all public items have documentation
lint-docs:
  cargo rustdoc --lib -- -D missing_docs -D rustdoc::broken_intra_doc_links

# Builds HTML docs for the sdk
build-docs:
  poetry run mkdocs build

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

# Run rust unit tests
test-rs:
  cargo test

# Generate type stubs from Rust code
generate-stubs:
  poetry run python ./scripts/generate_stubs.py
  just fmt

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

# Run full CI pipeline: format check, lint, type check, build, and test
ci: fmt-check readme-check lint-docs lint-check type-check install-package test-py test-rs
