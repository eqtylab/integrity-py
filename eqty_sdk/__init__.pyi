"""Type stubs for the eqty_sdk package."""

import eqty_sdk._rust as _rust
from eqty_sdk._rust import (
    get_cid_for_bytes,
    get_cid_for_json,
    get_cid_for_path,
    init,
    purge_blob_store,
    purge_statement_store,
    verify_statement,
    verify_vc,
)
from eqty_sdk.asset import (
    Agent,
    Asset,
    AssetType,
    Benchmark,
    BenchmarkResult,
    Binary,
    Certificate,
    Code,
    Configuration,
    Credential,
    Custom,
    Database,
    Dataset,
    Document,
    Guardrail,
    Media,
    Model,
    Prompt,
    Reasoning,
    Skill,
    SystemPrompt,
    Token,
    Tool,
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

CID = _rust.CID
DID = _rust.DID
SIGNER_ALGORITHMS = _rust.SIGNER_ALGORITHMS
UUID = _rust.UUID
Config = _rust.Config
Context = _rust.Context
Entity = _rust.Entity
Service = _rust.Service
Signer = _rust.Signer

_context_root_new = Context.new

def set_active_signer(signer: Signer) -> None:
    """Set the active signer used by higher-level SDK operations."""
    ...

__all__ = [
    "init",
    "get_cid_for_bytes",
    "get_cid_for_json",
    "get_cid_for_path",
    "purge_blob_store",
    "purge_statement_store",
    "Config",
    "verify_statement",
    "verify_vc",
    "Agent",
    "Asset",
    "AssetType",
    "Benchmark",
    "BenchmarkResult",
    "Binary",
    "Certificate",
    "Code",
    "Configuration",
    "Credential",
    "Custom",
    "Database",
    "Dataset",
    "Document",
    "Guardrail",
    "Media",
    "Model",
    "Prompt",
    "Reasoning",
    "Skill",
    "SystemPrompt",
    "Token",
    "Tool",
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
