# Eqty Python SDK

A Python SDK for tracking data provenance, asset lineage, and computation integrity. The SDK provides content-addressed storage and cryptographic verification for AI/ML workflows.

## Features

- **Asset Management**: Track datasets, models, code, documents, media, and custom asset types
- **Content Addressing**: Generate CIDs (Content Identifiers) for files, directories, and in-memory objects
- **Computation Tracking**: Capture input/output relationships for functions with the `@compute` decorator
- **Cryptographic Signing**: Sign statements with Ed25519 or via external notary services
- **Provenance Statements**: Create verifiable records of data lineage and transformations
- **Manifests**: Package and export provenance graphs for sharing or archival

## Installation

```sh
pip install eqty_sdk
```

## Quick Start

```python
import eqty_sdk
from eqty_sdk import Dataset, compute, init, Signer, set_active_signer, SIGNER_ALGORITHMS

# Initialize the SDK
init()

# Set up a signer for cryptographic verification
# Reuses the same signer on later runs if it already exists on disk.
signer = Signer.new("Default Signer", _load_if_exists=True)
set_active_signer(signer)

# Create a tracked dataset
dataset = Dataset.from_object({"key": "value"}, name="My Dataset")

# Track a computation with inputs and outputs
@compute(metadata={"name": "Process Data"})
def process(data: Dataset):
    result = transform(data.value)
    return Dataset.from_object(result, name="Processed Data")

output = process(dataset)
```

## Asset Types

The SDK supports various asset types:

| Type | Description |
|------|-------------|
| `Dataset` | Data files or in-memory data structures |
| `Model` | ML models and weights |
| `Code` | Source code and scripts |
| `Document` | Documents and text files |
| `Media` | Images, audio, and video |
| `Certificate` | Certificates and credentials |
| `Benchmark` | Benchmark definitions and results |
| `Custom` | User-defined asset types |

Each asset can be created from:
- A file path: `Dataset.from_path("./data.csv")`
- An object: `Dataset.from_object({"key": "value"})`
- A CID: `Dataset.from_cid("bafyrei...")`

## Development

### Prerequisites

- [just](https://github.com/casey/just) - Command runner
- [poetry](https://python-poetry.org/docs/) - Python dependency management
- Python >= 3.9
- Rust toolchain (for building the native extension)

### Setup

```sh
# Configure poetry and install dependencies
just install

# Enter the development shell
just dev
```

### Development Workflow

1. Make changes to Rust code in `./src/`
2. Make changes to Python code in `./eqty_sdk/`
3. Build and install the package locally:
   ```sh
   just install-package
   ```
   This compiles the Rust extension and generates type stubs for IDE support.

### Available Commands

The project uses [Just](https://github.com/casey/just) for common development tasks.

Run `just` to see all available commands.

```present just --list
Available recipes:
    build           # Build the Rust/Python wheel using maturin
    build-docs      # Builds HTML docs for the sdk
    ci              # Run full CI pipeline: format check, lint, type check, build, and test
    fix             # Auto-fix Rust clippy warnings
    fmt             # Auto-format code (Rust + Python)
    fmt-check       # Check code formatting without changes (Rust + Python)
    generate-stubs  # Generate type stubs from Rust code
    init            # Set up git hooks for prek
    install         # Install all Python dependencies via poetry
    install-package # Install the local build of the wheel into the venv
    lint            # Run linters and auto-fix issues (Rust clippy + Python ruff)
    lint-check      # Run linters without auto-fixing (Rust clippy + Python ruff)
    lint-docs       # Check that all public items have documentation
    readme-check    # Check if README.md is up to date with auto-generated content
    readme-update   # Update README.md with auto-generated content (Justfile commands, etc.)
    test-example-manifests # Run example scripts and compare normalized manifests to expected outputs
    test-py         # Run Python unit tests
    test-rs         # Run rust unit tests
    type-check      # Run mypy type checking on the Python SDK
```

### Project Structure

```
├── eqty_sdk/              # Python package exports and pure-Python helpers
│   ├── asset/             # Asset classes
│   └── compute/           # Compute decorators and helpers
├── src/                   # Rust implementation and PyO3 bindings
│   ├── indexer/           # SQLite-backed graph and statement indexing
│   ├── integrity_service/ # Integrity service client helpers
│   └── statements/        # Statement creation and registration bindings
├── tests/                 # Python unit tests
│   ├── 01_config_tests/
│   ├── asset_tests/
│   ├── computation_tests/
│   ├── context_tests/
│   ├── core_tests/
│   ├── indexer_tests/
│   ├── signer_tests/
│   └── statement_tests/
├── integration-tests/     # Integration test assets
├── examples/              # Usage examples
├── scripts/               # Development utilities
└── docs/                  # MkDocs documentation
```
