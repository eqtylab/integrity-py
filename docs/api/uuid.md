# UUID

A UUID is a stable random identifier for objects that are not content-addressed.

In `eqty_sdk`, UUIDs are useful for entities and references that need identity without being derived from file or object content. They show up in places such as `Entity`, contexts linked to Governance Studio projects, and low-level statements that need to identify something by persistent ID rather than by CID.

You should care about UUIDs when you are working with external system identifiers, entity-style records, or context/project mapping. If your workflow is mostly asset-based, you will usually work with CIDs more often than UUIDs.

::: eqty_sdk._rust.UUID
    options:
      members_order: source
