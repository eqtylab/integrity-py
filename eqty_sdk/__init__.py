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
    verify_statement_rdfc_cid,
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
from eqty_sdk.statements import ASSOCIATION_TYPES, Association

_context_root_new = Context.new


# This is about the cleanest way to keep user from doing Context.from_uuid(..).new()
# which would overwrite their from_uuid and just generate a new random Context
class _ContextNewDescriptor:
    def __get__(self, obj, owner):
        if obj is None:
            return _context_root_new

        def _invalid_instance_new(name: str):
            raise TypeError(
                "Calling .new(...) on a Context instance is ambiguous. "
                "Use Context.new(name) for a root context or "
                "Context.with_parent(ctx).new(name) for a child context."
            )

        return _invalid_instance_new


Context.new = _ContextNewDescriptor()  # type: ignore[method-assign]  # pyright: ignore[reportAttributeAccessIssue]


def set_active_signer(signer: Signer) -> None:
    """Set the active signer used by higher-level SDK operations."""
    _signer_module.set_active_signer(signer)


__all__ = [
    # Core
    "init",
    "get_cid_for_bytes",
    "get_cid_for_json",
    "get_cid_for_path",
    "verify_vc",
    "verify_statement_rdfc_cid",
    "purge_blob_store",
    "purge_statement_store",
    "Config",
    # Assets
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
