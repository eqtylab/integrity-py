# Assets

Assets are the main way to register inputs, outputs, documents, models, and other tracked objects in the SDK.

## Built-In Asset Types

The SDK includes these built-in asset categories:

--8<-- "docs/generated/built-in-asset-types.md"

These types all follow the same construction pattern.

!!! note "Use factory methods, not direct constructors"

    Do not instantiate typed assets directly (for example, do not call
    `Binary(...)` or `Dataset(...)`). Their displayed Python constructors are
    internal implementation details. Create assets with `from_object(...)`,
    `from_path(...)`, or `from_cid(...)` instead.

`Binary` is intended for compiled binary programs and similar executable artifacts that you want
to register as part of a lineage graph.

## Public constructors

Most built-in asset types support these constructors:

- `.from_object(...)`
- `.from_path(...)`
- `.from_cid(...)`
- `.with_context(ctx).from_object(...)`
- `.with_context(ctx).from_path(...)`
- `.with_context(ctx).from_cid(...)`

Conceptually:

- `from_object(...)`: hash and register an in-memory Python object
- `from_path(...)`: hash and register a file or directory on disk
- `from_cid(...)`: create an asset wrapper around an existing CID
- `with_context(...)`: do the same thing, but attach it to a specific context

## `Custom`

Use `Custom` when none of the built-in asset categories fit your domain object.

You can use the default custom type:

```python
from eqty_sdk import Custom

asset = Custom.from_object({"kind": "prompt-template"}, name="Prompt Template")
```

Or provide your own asset type label:

```python
from eqty_sdk import Custom

asset = Custom.from_object(
    {"kind": "feature-store-table"},
    asset_type="FeatureStoreTable",
    name="Customer Features",
)
```

The same pattern works with `from_path(...)` and `from_cid(...)`.

## Metadata Via `**kwargs`

Any extra keyword arguments passed to an asset constructor are stored as metadata on that asset.

Common examples include:

- `name`
- `description`
- domain-specific fields like `owner`, `source`, `version`, or `input`

```python
from eqty_sdk import Dataset

dataset = Dataset.from_object(
    [1, 2, 3],
    name="Training Rows",
    description="Rows used for the baseline experiment",
    owner="ml-team",
    version="2026-03-20",
)
```

Those values are included in the metadata statement the SDK creates for the asset.

## Serialization Notes

For `from_object(...)`, the SDK hashes a serialized representation of the object.

Built-in support includes:

- strings
- numbers
- lists
- dicts
- filesystem paths
- objects with `serialize_for_hashing()`

For a complex type that should be registered as its own content, implement
`serialize_for_hashing()` and return a stable `bytes` representation. The SDK
hashes those bytes to create the asset's CID:

```python
class PromptTemplate:
    def __init__(self, text: str):
        self.text = text

    def serialize_for_hashing(self) -> bytes:
        return self.text.encode("utf-8")
```

Use this with `Dataset.from_object(...)`, another typed asset's
`from_object(...)`, or the `Computation` builder's object methods.

## Adapt Runtime Values for `@compute`

`to_eqty_asset()` is different from `serialize_for_hashing()`. It is an adapter
for a value passed to a `@compute` function, and it must return an SDK `Asset`.
Use it when the runtime value is a handle or view over a different artifact that
should appear in lineage.

For example, a Spark DataFrame may be backed by a Parquet directory. The
compute function can keep receiving the DataFrame, while the lineage graph
records the on-disk dataset:

```python
from pathlib import Path

from eqty_sdk import Dataset


class ParquetBackedDataFrame:
    def __init__(self, dataframe, parquet_path: Path):
        self.dataframe = dataframe
        self.parquet_path = parquet_path

    def to_eqty_asset(self) -> Dataset:
        return Dataset.from_path(self.parquet_path, name="Training Data")
```

`to_eqty_asset()` is recognized only for `@compute` inputs. It does not hash the
wrapper object and is not used by `from_object(...)` or the `Computation`
builder. See [Spark Parquet Input](../examples/custom-serialize.md) for a
complete example.

## Factory Method Reference

The factory methods below are available on every built-in typed asset class,
including `Binary`, `Dataset`, `Document`, `Model`, and `Prompt`.

::: eqty_sdk.asset.asset.TypedAsset.from_object
    options:
      show_root_heading: true
      show_if_no_docstring: true

::: eqty_sdk.asset.asset.TypedAsset.from_path
    options:
      show_root_heading: true
      show_if_no_docstring: true

::: eqty_sdk.asset.asset.TypedAsset.from_cid
    options:
      show_root_heading: true
      show_if_no_docstring: true

::: eqty_sdk.asset.asset.TypedAsset.with_context
    options:
      show_root_heading: true
      show_if_no_docstring: true
