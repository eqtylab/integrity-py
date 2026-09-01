# Eqty Python SDK

Create signed, verifiable provenance for data, models, prompts, code, and Python computations.

The SDK records inputs, outputs, and execution relationships as an Integrity Manifest that you can inspect in [Eqty Explorer](https://explorer.eqtylab.io).

## Install

Python 3.10 or newer is required.

```bash
python -m pip install eqty_sdk
```

## Create your first manifest

This example registers an input dataset, records a computation, and exports the resulting manifest.

```python
from pathlib import Path

from eqty_sdk import Dataset, SIGNER_ALGORITHMS, Signer, compute, init, set_active_signer

# Initialize local SDK storage and the signer used for generated statements.
context = init().set_store_all_blobs(True).get_default_context()
set_active_signer(Signer.new(SIGNER_ALGORITHMS.SECP256R1))

input_data = Dataset.from_object({"customer_id": 42, "status": "active"})

@compute(metadata={"name": "Normalize customer status"})
def normalize(dataset: Dataset) -> Dataset:
    return Dataset.from_object({"customer_id": 42, "status": dataset.value["status"].upper()})

normalize(input_data)
context.export(Path("manifest.json"))
```

Import `manifest.json` into [Eqty Explorer](https://explorer.eqtylab.io) to inspect the assets, computation, and signed statements.

## Learn more

- [Quick start and guides](https://eqtylab.github.io/integrity-py/latest/)
- [Asset reference](https://eqtylab.github.io/integrity-py/latest/api/assets/)
- [Examples](https://eqtylab.github.io/integrity-py/latest/examples/)

For contributing to the SDK itself, see the repository's [developer README](https://github.com/eqtylab/integrity-py#readme).
