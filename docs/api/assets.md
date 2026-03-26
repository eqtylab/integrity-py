# Assets

Assets are the main way to register inputs, outputs, documents, models, and other tracked objects in the SDK.

## Built-In Asset Types

The SDK includes these built-in asset categories:

--8<-- "docs/generated/built-in-asset-types.md"

These types all follow the same construction pattern.

`Binary` is intended for compiled binary programs and similar executable artifacts that you want
to register as part of a lineage graph.

## Constructors

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
- compute inputs with `to_eqty_asset()`

If you have a complex type that should not be hashed directly, either:

- implement `serialize_for_hashing()` for stable hashing, or
- implement `to_eqty_asset()` and map the object to a more appropriate asset source

::: eqty_sdk.asset
    options:
      members_order: source
      filters:
        - "!^serialize_for_hashing$"
