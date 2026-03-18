"""Type stubs for the eqty_sdk package."""

import eqty_sdk._rust as _rust
import eqty_sdk.asset as _asset
import eqty_sdk.compute as _compute

class CID(_rust.CID): ...
class DID(_rust.DID): ...
class SIGNER_ALGORITHMS(_rust.SIGNER_ALGORITHMS): ...
class UUID(_rust.UUID): ...
class Config(_rust.Config): ...
class Context(_rust.Context): ...
class Entity(_rust.Entity): ...
class Service(_rust.Service): ...
class Signer(_rust.Signer): ...

get_cid_for_bytes = _rust.get_cid_for_bytes
get_cid_for_json = _rust.get_cid_for_json
get_cid_for_path = _rust.get_cid_for_path
init = _rust.init
purge_blob_store = _rust.purge_blob_store
purge_statement_store = _rust.purge_statement_store

Agent = _asset.Agent
Asset = _asset.Asset
AssetType = _asset.AssetType
Attribution = _asset.Attribution
Benchmark = _asset.Benchmark
BenchmarkResult = _asset.BenchmarkResult
Certificate = _asset.Certificate
Code = _asset.Code
Custom = _asset.Custom
Database = _asset.Database
Dataset = _asset.Dataset
Document = _asset.Document
Media = _asset.Media
Model = _asset.Model
Token = _asset.Token

Computation = _compute.Computation
Compute = _compute.Compute
compute = _compute.compute
from eqty_sdk.declaration import Declaration
from eqty_sdk.errors import (
    Error,
    UsageError,
)
from eqty_sdk.statements import ASSOCIATION_TYPES, Association

def set_active_signer(signer: str | Signer) -> None: ...

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
