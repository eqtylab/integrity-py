# Declaration

A declaration is a lightweight way to describe a value or object that should participate in provenance without forcing the SDK to infer everything automatically from the raw Python value.

You should care about declarations when you need to be explicit about how a function input or output is represented in the graph. They are useful in advanced compute workflows where a plain Python value is not descriptive enough on its own, or where you want to control the asset metadata and identity associated with that value.

Most users will rely on assets and `@compute` directly, but declarations become useful when you are shaping more customized lineage behavior around function boundaries.

::: eqty_sdk.Declaration
    options:
      members_order: source
