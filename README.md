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
signer = Signer.from_private_key(
    algorithm=SIGNER_ALGORITHMS.ED25519,
    private_key="<your-base64-encoded-private-key>",
)
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
    build                  # Build the project for native target
    build-wasm             # Build WebAssembly package with wasm-pack
    ci                     # Run all CI checks (format, build, lint, test)
    fix                    # Auto-fix clippy warnings where possible
    fmt                    # Format Rust code using rustfmt
    fmt-check              # Check if code is formatted correctly without modifying files
    lint                   # Run clippy lints to check code quality
    lint-docs              # Check that all public items have documentation (warnings only for now)
    pre-commit             # Run all prek pre-commit hooks on all files
    pre-push               # Called by git pre-push hook
    readme-check           # Check if README.md is up to date with auto-generated content
    readme-update          # Update README.md with auto-generated content (Justfile commands, etc.)
    test                   # Run unit tests with cargo
    test-wasm              # Run WASM tests in Node.js and browsers (Chrome, Firefox) Note: for macOS test Safari with --safari
    update-static-contexts # Rebuild static JSON-LD context files
```

### Project Structure

```
├── eqty_sdk/           # Python SDK source
│   ├── asset/          # Asset type definitions
│   ├── compute/        # Computation tracking
│   ├── config/         # Configuration management
│   ├── statements/     # Provenance statement types
│   └── types/          # Core type definitions
├── src/                # Rust source (PyO3 bindings)
│   └── statements/     # Rust statement implementations
├── tests/              # Python unit tests
└── examples/           # Usage examples
```
