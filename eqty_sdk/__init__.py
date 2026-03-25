from eqty_sdk._rust import (
    CID,
    DID,
    SIGNER_ALGORITHMS,
    UUID,
    Config,
    Context,
    Entity,
    Service,
    Signer,
    get_cid_for_bytes,
    get_cid_for_json,
    get_cid_for_path,
    init,
    purge_blob_store,
    purge_statement_store,
    signer as _signer_module,
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
from eqty_sdk.statements import ASSOCIATION_TYPES, Association


def set_active_signer(signer: Signer) -> None:
    """Set the active signer used by higher-level SDK operations."""
    _signer_module.set_active_signer(signer)


__all__ = [
    # Core
    "init",
    "get_cid_for_bytes",
    "get_cid_for_json",
    "get_cid_for_path",
    "purge_blob_store",
    "purge_statement_store",
    "Config",
    # Assets
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
    # Compute
    "compute",
    "Compute",
    "Computation",
    # Statements
    "Association",
    "ASSOCIATION_TYPES",
    # Context
    "Context",
    "Service",
    # Errors
    "Error",
    "UsageError",
    # Types
    "Declaration",
    "DID",
    "CID",
    "SIGNER_ALGORITHMS",
    "Signer",
    "set_active_signer",
    "Entity",
    "UUID",
]
