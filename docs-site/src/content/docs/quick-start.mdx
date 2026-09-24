# Quick Start

In about 10 minutes, you will install the SDK, run a small lineage-tracked workflow, export an Integrity Manifest, and open it in Explorer. Along the way, you will create a software DID signer, register data, and record a computation.

## 1. Install

The package is published on [PyPI](https://pypi.org/). Install it with:

```bash
python -m pip install eqty_sdk
```

Verify the installation:

```bash
python -c "import eqty_sdk; print(eqty_sdk.__file__)"
```

## 2. Run a complete example

Create `quick_start.py` with the following code, then run
`python quick_start.py`.

```python
--8<-- "examples/quick-start.py"
```

The script prints the formatted prompt and writes `manifests/quick-start.json`. `Signer.load_or_create(...)` gives the process a stable, local software key and DID. With that signer active, the SDK attributes and signs the statements it produces; rerunning the script uses the same DID rather than creating a new identity.

## 3. View the manifest in Explorer

In [Explorer](https://explorer.eqtylab.io), import or upload `manifests/quick-start.json`. Open the imported graph and select nodes to inspect their metadata, CIDs, statements, and signing identity.

The graph is a provenance map, not a control-flow trace:

- **Asset nodes** represent the content-addressed inputs and outputs, such as the system prompt, the user prompt, the temperature, the formatter's source code, and its result.
- **Compute nodes** represent an execution. Their incoming edges are the inputs used by that execution; their outgoing edges are the assets it produced.
- **Statement and metadata nodes** provide the assertions that connect those objects, including the DID-backed signatures.

Start at `Prompt Formatter`, follow its input edges to see what informed the result, then follow output edges to see what it created. Select an asset when you need its CID or metadata, and select a statement when you need to inspect the signed provenance assertion.

## Assets: type and source are separate choices

An asset type describes *what* content represents. The SDK includes types such as `Dataset`, `Model`, `Prompt`, `SystemPrompt`, `Code`, `Document`, `Agent`, `Tool`, `Configuration`, `Guardrail`, and `Custom`; see the complete [asset reference](api/assets.md#built-in-asset-types).

For nearly every type, choose one of three constructors based on *where* the content comes from:

```python
from eqty_sdk import CID, Dataset, Document, Model

rows = Dataset.from_object([{"id": 1, "text": "hello"}])
policy = Document.from_path("policies/retention.md")
known_model = Model.from_cid(
    CID("bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku")
)
```

- `from_object(...)` serializes in-memory content, computes its CID, and registers it.
- `from_path(...)` hashes a file or directory, computes its CID, and registers it.
- `from_cid(...)` registers a typed reference to content already identified by a CID; it does not have a local pre-image to store.

Use `Custom` with an `asset_type=` label when none of the built-in types fits.

## Compute: decorator or builder

The example uses `@compute`, the convenient choice when you want the SDK to capture a normal Python function's source, arguments, return value, and execution relationship.

Use the `Computation` builder when the work happens outside a Python function, or when you already have CIDs, paths, or objects for the inputs, output, and computation identity:

```python
from eqty_sdk import Computation

(Computation.new(name="External Training Job", _store=False)
    .add_input_path("data/train.csv")
    .add_output_path("artifacts/model.bin")
    .set_computation_object({"job": "train-v1", "runtime": "remote"})
    .finalize())
```

Both approaches create the same essential lineage: the computation links its input CIDs to its output CIDs and receives metadata and signed statements. The builder's `_store` setting applies to every object and path it hashes.

## Decide whether to retain pre-image blobs

Every object or path registration computes a CID whether or not its original bytes are retained. Retaining the bytes (the **pre-image blob**) makes the content available for later manifest/service workflows, but it can consume disk space and may be inappropriate for sensitive or large data.

Choose the default for the whole SDK configuration:

```python
cfg = init()
cfg.set_store_all_blobs(True)   # retain every newly hashed pre-image by default
```

Set it to `True` when you plan to register blobs with a service, need local reproducibility, or want the content available alongside its manifest. Set it to `False` when only CIDs and provenance are needed locally, or when retaining the source bytes would be too costly or sensitive.

Override that default for an individual data registration with `_store`:

```python
public_summary = Dataset.from_object({"count": 42}, _store=True)
sensitive_rows = Dataset.from_path("private/records.csv", _store=False)
```

The same override is available when you register a compute with either the builder or decorator. For a decorator, it applies to the code, captured inputs, and outputs it hashes:

```python
@compute(metadata={"name": "Sensitive transform"}, _store=False)
def transform(rows: Dataset) -> str:
    return "redacted summary"
```

Use per-call `_store=True` to preserve an important, non-sensitive input or result when the global default is `False`; use `_store=False` to opt a specific sensitive or very large registration out when the global default is `True`. `from_cid(...)` has no `_store` option because it only refers to an existing CID and does not provide bytes for the SDK to retain.

For full configuration and service-registration details, see
[Init and Config](api/init-and-config.md) and
[Service Registering](examples/service-registering.md).
