"""Type stubs for the eqty_sdk package."""

import eqty_sdk._rust as _rust
from eqty_sdk._rust import (
    get_cid_for_bytes,
    get_cid_for_json,
    get_cid_for_path,
    init,
    purge_blob_store,
    purge_statement_store,
    signer,
)
from eqty_sdk.asset import (
    Agent,
    Asset,
    AssetType,
    Attribution,
    Benchmark,
    BenchmarkResult,
    Certificate,
    Code,
    Custom,
    Database,
    Dataset,
    Document,
    Media,
    Model,
    Token,
)
from eqty_sdk.compute import (
    Computation,
    Compute,
    compute,
)
from eqty_sdk.declaration import Declaration
from eqty_sdk.errors import (
    Error,
    UsageError,
)
from eqty_sdk.statements import (
    ASSOCIATION_TYPES,
    Association,
)

class CID(_rust.CID): ...
class DID(_rust.DID): ...
class SIGNER_ALGORITHMS(_rust.SIGNER_ALGORITHMS): ...
class UUID(_rust.UUID): ...
class Config(_rust.Config): ...
class Context(_rust.Context): ...
class Entity(_rust.Entity): ...
class Service(_rust.Service): ...
class Signer(_rust.Signer): ...

set_active_signer = signer.set_active_signer

__all__ = [
    "init",
    "get_cid_for_bytes",
    "get_cid_for_json",
    "get_cid_for_path",
    "purge_blob_store",
    "purge_statement_store",
    "Config",
    "Agent",
    "Asset",
    "AssetType",
    "Attribution",
    "Benchmark",
    "BenchmarkResult",
    "Certificate",
    "Code",
    "Custom",
    "Database",
    "Dataset",
    "Document",
    "Media",
    "Model",
    "Token",
    "compute",
    "Compute",
    "Computation",
    "Association",
    "ASSOCIATION_TYPES",
    "Context",
    "Service",
    "Error",
    "UsageError",
    "Declaration",
    "DID",
    "CID",
    "SIGNER_ALGORITHMS",
    "Signer",
    "set_active_signer",
    "Entity",
    "UUID",
]
