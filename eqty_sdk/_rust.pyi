"""Type stubs for the eqty_sdk._rust module."""
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Union
from os import PathLike

def init(custom_dir: Optional[PathLike[str]] = None) -> Any:
    """Initializes the sdk config. Must be called before setting individual config values"""
    ...

def get_cid_for_bytes(data: Any, store: Optional[bool] = None) -> str:
    """Calculates and returns the CID for the provided bytes."""
    ...

def get_cid_for_path(path: PathLike[str], store: Optional[bool] = None) -> str:
    """Resolves the provided path and reads the file or directory to calculate the CID."""
    ...

def maybe_create_model_signing_statement(collection_cid: str, model_signing_name: str, is_dir: bool) -> None:
    """Creates a model signing statement if enabled in config and the asset is a directory."""
    ...

class Asset:
    """Asset wrapper for data/metadata registration."""
    @property
    def statement_ids(self) -> List[str]:
        """Statement IDs created for this asset."""
        ...

    @property
    def cid(self) -> str:
        """Content identifier."""
        ...

    @property
    def asset_type(self) -> str:
        """Asset type."""
        ...

    @property
    def name(self) -> str:
        """Asset name."""
        ...

    @property
    def value(self) -> Any:
        """Underlying value."""
        ...

    def __init__(self, obj: Any, asset_type: Any, cid: str, is_dir: bool) -> None:
        ...

    @staticmethod
    def _from_object(obj: Any, asset_type: Any, ctx: Optional[Graph], store: Optional[bool]) -> Asset:
        ...

    @staticmethod
    def _from_path(path: PathLike[str], asset_type: Any, ctx: Optional[Graph], store: Optional[bool]) -> Asset:
        ...

    @staticmethod
    def _from_cid(cid: str, asset_type: Any, ctx: Optional[Graph]) -> Asset:
        ...

    @staticmethod
    def _factory_with_context(ctx: Graph, asset_type: Any) -> Any:
        ...

    def add_declaration(self, declaration: Any) -> Asset:
        ...


class Canon:
    """Canonicalization options."""
    RDFC1: Canon
    JSONJCS: Canon


class Cid:
    """A simple wrapper around a content identifier (CID) string."""
    @property
    def cid(self) -> str:
        """Get the CID string."""
        ...

    def __init__(self, cid: str) -> None:
        ...

    def __str__(self) -> str:
        ...

    def __repr__(self) -> str:
        ...


class CidResult:
    """Result of CID computation."""
    @property
    def cid(self) -> str:
        """Get the CID string."""
        ...

    @property
    def blob(self) -> bytes:
        """Get the binary blob data."""
        ...


class Config:
    """Global configuration handle."""
    def set_integrity_service_url(self, url: str) -> Config:
        ...

    def set_hashing_config(self, multithread: Optional[bool], memory_map: Optional[bool]) -> Config:
        ...

    def set_cid_ignore_rules(self, include_hidden_files: Optional[bool], gitignore: Optional[bool], include_symlinks: Optional[bool]) -> Config:
        ...

    def set_generate_model_signing_signatures(self, enable: bool) -> Config:
        ...

    def set_store_all_blobs(self, value: bool) -> Config:
        ...

    def set_default_graph(self, graph: Graph) -> Config:
        ...


class Declaration:
    """Declaration for governance statements."""
    @property
    def subject_line(self) -> str:
        """Subject line for the declaration."""
        ...

    @property
    def statement(self) -> str:
        """Declaration statement."""
        ...

    @property
    def submitted_at(self) -> Optional[str]:
        """Submission timestamp."""
        ...

    @property
    def submitted_by(self) -> Optional[str]:
        """DID key of submitter."""
        ...

    @property
    def control_cid(self) -> List[str]:
        """Control CIDs."""
        ...

    @property
    def attachment_cid(self) -> List[str]:
        """Attachment CIDs."""
        ...

    @property
    def extra(self) -> Dict[str, str]:
        """Additional metadata."""
        ...

    def __init__(self, subject_line: str, statement: str) -> None:
        ...

    @staticmethod
    def new(subject_line: str, statement: str) -> Declaration:
        ...

    def add_attachment_cid(self, cid: str) -> Declaration:
        ...

    def add_control_cid(self, cid: str) -> Declaration:
        ...

    def add_extra(self, key: str, val: str) -> Declaration:
        ...

    def finalize(self) -> Declaration:
        ...

    def cid(self) -> str:
        ...

    def to_dict(self) -> Dict[str, Any]:
        ...

    def to_json(self) -> str:
        ...


class Did:
    """DID registration helper."""
    @property
    def ctx(self) -> Graph:
        """Graph context for the DID."""
        ...

    @property
    def statement_ids(self) -> List[str]:
        """Statement IDs created for this DID."""
        ...

    def __init__(self, ctx: Graph, did: str, signer: Optional[Signer]) -> None:
        ...

    @staticmethod
    def from_signer(signer: Signer) -> Did:
        ...

    @staticmethod
    def from_did_string(did: str) -> Did:
        ...

    @staticmethod
    def with_context(ctx: Graph) -> DidFactory:
        ...


class DidFactory:
    """Factory for creating DID objects with a fixed context."""
    def build_from_signer(self, signer: Signer) -> Did:
        ...

    def build_from_did_string(self, did: str) -> Did:
        ...


class DirCidResult:
    """Result of directory CID computation."""
    @property
    def collection(self) -> CidResult:
        """Get the collection CID result."""
        ...

    @property
    def meta(self) -> CidResult:
        """Get the metadata CID result."""
        ...

    @property
    def file_hashes(self) -> List[Tuple[str, str]]:
        """Get list of (filename, CID) tuples."""
        ...


class Entity:
    """Represents an unhashed entity with a UUID identifier."""
    @property
    def uuid(self) -> str:
        """Get the UUID string."""
        ...

    def __init__(self, uuid: str) -> None:
        ...

    @staticmethod
    def generate() -> Entity:
        ...

    @staticmethod
    def from_uuid(uuid: str) -> Entity:
        ...

    def __str__(self) -> str:
        ...

    def __repr__(self) -> str:
        ...


class Graph:
    """Graph for organizing statements."""
    @property
    def id(self) -> Any:
        """UUID of the graph."""
        ...

    @property
    def name(self) -> str:
        """Name of the graph."""
        ...

    @property
    def parent(self) -> Optional[Any]:
        """UUID of the parent graph."""
        ...

    def __init__(self, id: Any, name: str) -> None:
        ...

    @staticmethod
    def from_parent(id: Any, name: str, graph: Graph) -> Graph:
        ...


class Manifest:
    """Manifest utilities and representation."""
    @property
    def manifest_str(self) -> str:
        """Manifest JSON string."""
        ...

    def __init__(self, manifest: str) -> None:
        ...

    @staticmethod
    def from_statements(statements: Any, include_context: bool) -> Manifest:
        ...

    def export(self, file: PathLike[str]) -> None:
        ...

    @staticmethod
    def import_manifest(manifest: Any) -> None:
        ...


class Metadata:
    """Metadata for subject descriptions."""
    def __init__(self) -> None:
        ...

    def __getattr__(self, attr: str) -> Any:
        ...

    def to_dict(self) -> Dict[str, Any]:
        ...

    def to_json_str(self) -> str:
        ...

    def create_statement(self, subject_cid: str, skip_proof: bool) -> List[str]:
        ...


class Signer:
    """Python wrapper for Rust signer."""
    @property
    def name(self) -> str:
        """Get the signer name."""
        ...

    @property
    def did_key(self) -> str:
        """Get the DID key."""
        ...


class SIGNER_ALGORITHMS:
    """Signer algorithm constants."""
    ED25519: SignerAlgorithms
    SECP256K1: SignerAlgorithms
    SECP256R1: SignerAlgorithms


# Manifest module
class manifest:
    Manifest: type[Manifest]

    @staticmethod
    def generate(statements: List[Any], blobs_dir: PathLike[str], include_context: Optional[bool] = None) -> str:
        """ # Arguments * `py` - Python interpreter reference * `graphs` - Python list of graph dictionaries, each containing 'id', 'name', 'parent', and 'statements' * `blobs_dir` - Path to directory containing blob files referenced by statements * `include_context` - Whether to include context information in the manifest (default: false)  # Returns * `PyResult<String>` - JSON string representation of the manifest, or error on failure"""
        ...

    @staticmethod
    def merge(a: str, b: str) -> str:
        """Merges the manifests `a` and `b` and returns the merged manifest."""
        ...

    @staticmethod
    def register(manifest: str, api_key: Optional[str] = None) -> None:
        """Register the manfiest with integrity platform"""
        ...


# Entity module
class entity:
    Entity: type[Entity]

    @staticmethod
    def create_entity(metadata_json: str, skip_proof: Optional[bool] = None, timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> Any:
        """ # Arguments * `metadata_json` - JSON string containing metadata to associate with the entity * `skip_proof` - If true, skip creating a VC statement * `timestamp` - Optional timestamp for statements * `graph_id` - Optional graph ID to register statements to  # Returns Tuple of (Entity, list of statement IDs)"""
        ...

    @staticmethod
    def create_entity_from_uuid(uuid: str, metadata_json: str, skip_proof: Optional[bool] = None, timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> Any:
        """# Arguments * `uuid` - UUID string for the entity * `metadata_json` - JSON string containing metadata to associate with the entity * `skip_proof` - If true, skip creating a VC statement * `timestamp` - Optional timestamp for statements * `graph_id` - Optional graph ID to register statements to  # Returns Tuple of (Entity, list of statement IDs)"""
        ...


# Cid module
class cid:
    Cid: type[Cid]
    CidResult: type[CidResult]
    DirCidResult: type[DirCidResult]
    Canon: type[Canon]

    @staticmethod
    def compute_cid_for_directory(path: PathLike[str]) -> Any:
        """Compute CID for a directory at `path`."""
        ...

    @staticmethod
    def compute_cid_for_file(path: PathLike[str]) -> Any:
        """Compute CID for a file `path`."""
        ...

    @staticmethod
    def compute_cid_for_bytes(bytes: Any) -> str:
        """Compute CID for provided bytes."""
        ...


# Stream module
class stream:
    @staticmethod
    def create(input_cids: List[str], operated_by: Optional[str] = None, executed_on: Optional[str] = None, timestamp: Optional[str] = None) -> Any:
        """creates a new computation stream"""
        ...

    @staticmethod
    def update(id: str, chunk: bytes) -> Any:
        """updates an existing computation stream with new data"""
        ...

    @staticmethod
    def finalize(id: str, static_output_cids: Optional[List[str]] = None, graph_id: Optional[Any] = None) -> Any:
        """Finalizes the computation stream and creates the ComputationStatement and (optionally) the VC and VCStatement returns the CID of the computation statement"""
        ...


# Signer module
class signer:
    Signer: type[Signer]
    SIGNER_ALGORITHMS: type[SIGNER_ALGORITHMS]

    @staticmethod
    def create_new_signer(key_type: str, name: Optional[Any] = None) -> Any:
        """Creates a new local signer with a randomly generated key.  # Arguments * `name` - Optional name for the signer (uses DID key if not provided) * `key_type` - Type of cryptographic key to generate (SECP256K1, SECP256R1, ED25519)"""
        ...

    @staticmethod
    def create_signer_from_private_key(key: str, key_type: str, name: Optional[Any] = None) -> Any:
        """Creates a signer from an existing base64-encoded private key.  # Arguments * `key` - Base64-encoded private key bytes * `key_type` - Type of cryptographic key (SECP256K1, SECP256R1, ED25519) * `name` - Optional name for the signer (uses DID key if not provided)"""
        ...

    @staticmethod
    def create_vcomp_signer(url: str, pub_key: Optional[str] = None) -> Any:
        """Creates a VComp notary signer for TEE-based remote signing.  # Arguments * `name` - Name to assign to the signer * `url` - VComp notary service URL * `key_type` - Type of key (currently only SECP256R1 is supported) * `pub_key` - Optional public key for the signer """
        ...

    @staticmethod
    def create_auth_service_signer(url: str, api_key: str) -> Any:
        """Creates an Auth Service-based signer for remote signing operations.  # Arguments * `name` - Name to assign to the signer * `url` -  Auth Service API endpoint URL * `api_key` - API key for authentication with the Auth Service"""
        ...

    @staticmethod
    def create_yubihsm2_signer(auth_key_id: int, signing_key_id: int, password: str) -> Any:
        """Creates and configures a YubiHSM2 hardware security module signer.  # Arguments * `name` - Name to assign to the signer * `auth_key_id` - Authentication key ID for YubiHSM2 * `signing_key_id` - Signing key ID for YubiHSM2 * `password` - Password for YubiHSM2 authentication"""
        ...

    @staticmethod
    def set_active_signer(signer: Any) -> None:
        """Sets the active signer by name or signer instance.  # Arguments * `signer` - Signer name string or Signer instance"""
        ...


# Statements module
class statements:
    @staticmethod
    def retrieve_graph(graph_ids: List[Any]) -> Any:
        """Retrieve statements for multiple graph IDs.  Args: graph_ids: List of graph UUIDs to retrieve graphs for  Returns: List of statements"""
        ...

    @staticmethod
    def register_statement(statement_json: str) -> None:
        """Register a statement from JSON string to the default graph."""
        ...

    @staticmethod
    def register_statement_to_graph(statement_id: str, graph_id: str) -> None:
        """Associate an existing statement with a graph."""
        ...

    @staticmethod
    def add_metadata_statement(subject: str, metadata: str, skip_proof: Optional[bool] = None, graph_id: Optional[Any] = None) -> List[str]:
        """Add Metadata Statement."""
        ...

    @staticmethod
    def add_data_statement(data: List[str], skip_proof: Optional[bool] = None, graph_id: Optional[Any] = None) -> List[str]:
        """Add Data Statement."""
        ...

    @staticmethod
    def add_governance_statement(subject: str, document: str, skip_proof: Optional[bool] = None, graph_id: Optional[Any] = None) -> List[str]:
        """Add Governance Statement."""
        ...

    @staticmethod
    def add_computation_statement(inputs: List[str], outputs: List[str], computation: Optional[str] = None, operated_by: Optional[str] = None, executed_on: Optional[str] = None, skip_proof: Optional[bool] = None, graph_id: Optional[Any] = None) -> List[str]:
        """Add Computation Statement."""
        ...

    @staticmethod
    def create_model_signing_statement(collection_cid: str, blobs_dir: PathLike[str], model_signing_name: str, allow_symlinks: bool, ignore_paths: List[str], timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> str:
        """Create Model Signing Statement."""
        ...

    @staticmethod
    def add_association_statement(subject: str, association: str, skip_proof: Optional[bool] = None, graph_id: Optional[Any] = None) -> List[str]:
        """Add Association Statement."""
        ...

    @staticmethod
    def add_entity_statement(entity: str, skip_proof: Optional[bool] = None, graph_id: Optional[Any] = None) -> List[str]:
        """Add Entity Statement."""
        ...

    @staticmethod
    def add_did_statement(did: str, skip_proof: Optional[bool] = None, graph_id: Optional[Any] = None) -> List[str]:
        """Add Did Statement."""
        ...

    @staticmethod
    def add_storage_statement(data: str, stored_on: str, operated_by: Optional[str] = None, skip_proof: Optional[bool] = None, graph_id: Optional[Any] = None) -> List[str]:
        """Add Storage Statement."""
        ...

    @staticmethod
    def create_vc_statement(subject: str, timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> str:
        """Create Vc Statement."""
        ...

