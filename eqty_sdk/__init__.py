from pathlib import Path
from typing import Optional, cast

from eqty_sdk._rust import (
    CID,
    DID,
    SIGNER_ALGORITHMS,
    UUID,
    Config as _Config,
    Declaration,
    Entity,
    Graph as Context,
    Service,
    Signer,
    get_cid_for_bytes,
    get_cid_for_path,
    init as _init,
    purge_blob_store,
    purge_statement_store,
    signer,
)
from eqty_sdk.asset import (
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
from eqty_sdk.errors import (
    AuthenticationError,
    Error,
    ExternalError,
    UnsupportedError,
    UsageError,
)

set_active_signer = signer.set_active_signer


def init(app_dir: Optional[Path] = None) -> _Config:
    """Initialize the SDK and return the config instance."""
    cfg = _init(app_dir)
    return cast(_Config, cfg)


__all__ = [
    # Core
    "init",
    "get_cid_for_bytes",
    "get_cid_for_path",
    "purge_blob_store",
    "purge_statement_store",
    # Assets
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
    # Context
    "Context",
    "Service",
    # Errors
    "AuthenticationError",
    "Error",
    "ExternalError",
    "UsageError",
    "UnsupportedError",
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
