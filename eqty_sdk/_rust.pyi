"""Type stubs for the eqty_sdk._rust module."""

import eqty_sdk
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Union
import uuid
from os import PathLike

def init(
    custom_dir: Optional[PathLike[str]] = None, default_context: Optional[Graph] = None
) -> Config:
    """Initializes the sdk config. Must be called before setting individual config values"""
    ...

def get_cid_for_bytes(data: bytes, store: Optional[bool] = None) -> str:
    """Calculates and returns the CID for the provided bytes."""
    ...

def get_cid_for_path(path: PathLike[str], store: Optional[bool] = None) -> str:
    """Resolves the provided path and reads the file or directory to calculate the CID."""
    ...

def maybe_create_model_signing_statement(
    _collection_cid: str, _model_signing_name: str, _is_dir: bool
) -> None:
    """Creates a model signing statement if enabled in config and the asset is a directory."""
    ...

class CID:
    """A simple wrapper around a content identifier (CID) string.  Provides a typed wrapper for CID strings with property access and string conversion."""
    @property
    def cid(self) -> str: ...
    def __init__(self, cid: str) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class Canon:
    """Canonicalization algorithm for computing content identifiers."""

    RDFC1: Canon
    JSONJCS: Canon

class CidResult:
    """Result of computing a content identifier (CID).  Contains both the CID string and the raw blob data."""
    @property
    def cid(self) -> str: ...
    @property
    def blob(self) -> bytes: ...

class Config:
    """Global application config containing configuration and state.  The config stores application-wide settings including storage directories, hashing preferences, and file filtering rules."""
    def set_hashing_config(
        self, multithread: Optional[bool] = None, memory_map: Optional[bool] = None
    ) -> Config: ...
    def set_cid_ignore_rules(
        self,
        include_hidden_files: Optional[bool] = None,
        gitignore: Optional[bool] = None,
        include_symlinks: Optional[bool] = None,
    ) -> Config: ...
    def set_generate_model_signing_signatures(self, enable: bool) -> Config: ...
    def set_store_all_blobs(self, value: bool) -> Config: ...

class DID:
    @property
    def ctx(self) -> Any: ...
    @property
    def statement_ids(self) -> List[str]: ...
    def __init__(
        self, ctx: Graph, did: str, signer: Optional[Signer] = None, **kwargs: Any
    ) -> None: ...
    @staticmethod
    def from_signer(signer: Signer, **kwargs: Any) -> DID: ...
    @staticmethod
    def from_did_string(did: str, **kwargs: Any) -> DID: ...
    @staticmethod
    def with_context(ctx: Graph) -> DidFactory: ...

class Declaration:
    @property
    def subject_line(self) -> str: ...
    @property
    def statement(self) -> str: ...
    @property
    def submitted_at(self) -> Optional[str]: ...
    @property
    def submitted_by(self) -> Optional[str]: ...
    @property
    def control_cid(self) -> List[str]: ...
    @property
    def attachment_cid(self) -> List[str]: ...
    @property
    def extra(self) -> Any: ...
    def __init__(self, subject_line: str, statement: str) -> None: ...
    @staticmethod
    def new(subject_line: str, statement: str) -> Declaration: ...
    def add_attachment_cid(self, cid: str) -> Declaration: ...
    def add_control_cid(self, cid: str) -> Declaration: ...
    def add_extra(self, key: str, val: str) -> Declaration: ...
    def finalize(self) -> Declaration: ...
    def cid(self) -> str: ...
    def to_dict(self) -> Any: ...
    def to_json(self) -> str: ...

class DidFactory:
    def build_from_signer(self, signer: Signer, **kwargs: Any) -> DID: ...
    def build_from_did_string(self, did: str, **kwargs: Any) -> DID: ...

class DirCidResult:
    """Result of computing a CID for a directory.  Contains the collection CID, metadata CID, and individual file hashes."""
    @property
    def collection(self) -> CidResult: ...
    @property
    def meta(self) -> CidResult: ...
    @property
    def file_hashes(self) -> List[Tuple[str, str]]: ...

class Entity:
    """Represents an unhashed entity with a UUID identifier.  Entities are used to represent objects that don't have a content-based identifier (CID) but need a unique identifier for tracking purposes."""
    @property
    def uuid(self) -> str: ...
    def __init__(self, uuid: str) -> None:
        """Create a new Entity w"""
        ...

    @staticmethod
    def generate() -> Entity:
        """Create a new Entity with"""
        ...

    @staticmethod
    def from_uuid(uuid: str) -> Entity:
        """Create an"""
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class Graph:
    """A graph structure for organizing related statements hierarchically.  Graphs group statements together with optional parent-child relationships, enabling versioning and organizational structure for lineage data."""
    @property
    def id(self) -> uuid.UUID:
        """Unique identifier for this graph"""
        ...

    @property
    def name(self) -> str:
        """Human-readable name for this graph"""
        ...

    @property
    def parent(self) -> Optional[uuid.UUID]:
        """Optional parent graph ID for hierarchical organization"""
        ...

    @staticmethod
    def new(name: str) -> Graph: ...
    @staticmethod
    def from_parent(parent: Graph) -> GraphFactory: ...
    @staticmethod
    def from_uuid(project_id: uuid.UUID) -> GraphFactory: ...
    def register(self, service: Service) -> None: ...
    def __str__(self) -> str: ...

class GraphFactory:
    def new(self, name: str) -> Graph: ...

class Manifest:
    @property
    def manifest_str(self) -> str: ...
    def __init__(self, manifest: str) -> None: ...
    def from_statements(
        self, statements: Any, include_context: Optional[bool] = None
    ) -> Manifest: ...
    def export(self, file: PathLike[str]) -> None: ...
    def import_manifest(self, manifest: Any) -> None: ...
    @staticmethod
    def merge(a: str, b: str) -> str: ...

class SIGNER_ALGORITHMS:
    ED25519: str
    SECP256K1: str
    SECP256R1: str

class Service:
    """Service for connecting to the Integrity Service API."""
    @property
    def base_path(self) -> str:
        """Base URL path for the API (e.g., `https://api.example.com`)."""
        ...

    @staticmethod
    def new(url: str, api_key: Optional[str] = None) -> Service: ...

class Signer:
    """Python-exposed signer information.  Contains the name and DID key of a cryptographic signer."""
    @property
    def name(self) -> str:
        """Returns the human-readable name of the signer.  # Returns *"""
        ...

    @property
    def did_key(self) -> str:
        """Returns the DID key of the signer.  # Returns * `&str` - T"""
        ...

    def __init__(self, name: str, did_key: str) -> None: ...
    @staticmethod
    def new(algorithm: Optional[Any] = None) -> Signer: ...
    @staticmethod
    def vcomp_notary(url: Optional[str] = None) -> Signer: ...
    @staticmethod
    def auth_service(url: str) -> Signer: ...
    @staticmethod
    def yubihsm2(auth_key_id: int, signing_key_id: int, password: str) -> Signer: ...
    @staticmethod
    def from_private_key(algorithm: Any, private_key: str) -> Signer: ...

# Cid module
class cid:
    Canon: type[Canon]
    DirCidResult: type[DirCidResult]
    CidResult: type[CidResult]
    CID: type[CID]

    @staticmethod
    def compute_cid_for_directory(path: PathLike[str]) -> eqty_sdk._rust.DirCidResult:
        """Compute CID for a directory at `path`."""
        ...

    @staticmethod
    def compute_cid_for_file(path: PathLike[str]) -> eqty_sdk._rust.CidResult:
        """Compute CID for a file `path`."""
        ...

    @staticmethod
    def compute_cid_for_bytes(bytes: bytes) -> str:
        """Compute CID for provided bytes."""
        ...

# Entity module
class entity:
    Entity: type[Entity]

    @staticmethod
    def create_entity(
        metadata_json: str,
        skip_proof: Optional[bool] = None,
        timestamp: Optional[str] = None,
        graph_id: Optional[uuid.UUID] = None,
    ) -> Tuple[eqty_sdk._rust.Entity, Any]:
        """# Arguments * `metadata_json` - JSON string containing metadata to associate with the entity * `skip_proof` - If true, skip creating a VC statement * `timestamp` - Optional timestamp for statements * `graph_id` - Optional graph ID to register statements to  # Returns Tuple of (Entity, list of statement IDs)"""
        ...

    @staticmethod
    def create_entity_from_uuid(
        uuid: str,
        metadata_json: str,
        skip_proof: Optional[bool] = None,
        timestamp: Optional[str] = None,
        graph_id: Optional[uuid.UUID] = None,
    ) -> Tuple[eqty_sdk._rust.Entity, Any]:
        """# Arguments * `uuid` - UUID string for the entity * `metadata_json` - JSON string containing metadata to associate with the entity * `skip_proof` - If true, skip creating a VC statement * `timestamp` - Optional timestamp for statements * `graph_id` - Optional graph ID to register statements to  # Returns Tuple of (Entity, list of statement IDs)"""
        ...

# Manifest module
class manifest:
    Manifest: type[Manifest]

    @staticmethod
    def generate(statements: List[Any], include_context: Optional[bool] = None) -> str:
        """Generates an integrity graph manifest JSON string from multiple graphs.  # Arguments * `py` - Python interpreter reference * `graphs` - Python list of graph dictionaries, each containing 'id', 'name', 'parent', and 'statements' * `include_context` - Whether to include context information in the manifest (default: false)  # Returns * `PyResult<String>` - JSON string representation of the manifest, or error on failure"""
        ...

    @staticmethod
    def merge(a: str, b: str) -> str:
        """Merges the manifests `a` and `b` and returns the merged manifest."""
        ...

# Signer module
class signer:
    Signer: type[Signer]
    SIGNER_ALGORITHMS: type[SIGNER_ALGORITHMS]

    @staticmethod
    def create_new_signer(key_type: str, name: Optional[str] = None) -> eqty_sdk._rust.Signer:
        """Creates a new local signer with a randomly generated key.  # Arguments * `name` - Optional name for the signer (uses DID key if not provided) * `key_type` - Type of cryptographic key to generate (SECP256K1, SECP256R1, ED25519)"""
        ...

    @staticmethod
    def create_signer_from_private_key(
        key: str, key_type: str, name: Optional[str] = None
    ) -> eqty_sdk._rust.Signer:
        """Creates a signer from an existing base64-encoded private key.  # Arguments * `key` - Base64-encoded private key bytes * `key_type` - Type of cryptographic key (SECP256K1, SECP256R1, ED25519) * `name` - Optional name for the signer (uses DID key if not provided)"""
        ...

    @staticmethod
    def create_vcomp_signer(url: str, pub_key: Optional[str] = None) -> eqty_sdk._rust.Signer:
        """Creates a VComp notary signer for TEE-based remote signing.  # Arguments * `name` - Name to assign to the signer * `url` - VComp notary service URL * `key_type` - Type of key (currently only SECP256R1 is supported) * `pub_key` - Optional public key for the signer"""
        ...

    @staticmethod
    def create_yubihsm2_signer(
        auth_key_id: int, signing_key_id: int, password: str
    ) -> eqty_sdk._rust.Signer:
        """Creates and configures a YubiHSM2 hardware security module signer.  # Arguments * `name` - Name to assign to the signer * `auth_key_id` - Authentication key ID for YubiHSM2 * `signing_key_id` - Signing key ID for YubiHSM2 * `password` - Password for YubiHSM2 authentication"""
        ...

    @staticmethod
    def create_auth_service_signer(url: str, api_key: str) -> eqty_sdk._rust.Signer:
        """Creates an Auth Service-based signer for remote signing operations.  # Arguments * `name` - Name to assign to the signer * `url` -  Auth Service API endpoint URL * `api_key` - API key for authentication with the Auth Service"""
        ...

    @staticmethod
    def set_active_signer(signer: Any) -> None:
        """Sets the active signer by name or signer instance.  # Arguments * `signer` - Signer name string or Signer instance"""
        ...

# Statements module
class statements:
    @staticmethod
    def add_association_statement(
        subject: str,
        association: str,
        *,
        skip_proof: Optional[bool] = None,
        graph_id: Optional[uuid.UUID] = None,
    ) -> List[str]: ...
    @staticmethod
    def add_computation_statement(
        inputs: List[str],
        outputs: List[str],
        computation: Optional[str] = None,
        *,
        operated_by: Optional[str] = None,
        executed_on: Optional[str] = None,
        skip_proof: Optional[bool] = None,
        graph_id: Optional[uuid.UUID] = None,
    ) -> List[str]: ...
    @staticmethod
    def add_data_statement(
        data: List[str], *, skip_proof: Optional[bool] = None, graph_id: Optional[uuid.UUID] = None
    ) -> List[str]: ...
    @staticmethod
    def add_did_statement(
        did: str, *, skip_proof: Optional[bool] = None, graph_id: Optional[uuid.UUID] = None
    ) -> List[str]: ...
    @staticmethod
    def add_entity_statement(
        entity: str, *, skip_proof: Optional[bool] = None, graph_id: Optional[uuid.UUID] = None
    ) -> List[str]: ...
    @staticmethod
    def add_governance_statement(
        subject: str,
        document: str,
        *,
        skip_proof: Optional[bool] = None,
        graph_id: Optional[uuid.UUID] = None,
    ) -> List[str]: ...
    @staticmethod
    def add_metadata_statement(
        subject: str,
        metadata: str,
        *,
        skip_proof: Optional[bool] = None,
        graph_id: Optional[uuid.UUID] = None,
    ) -> List[str]: ...
    @staticmethod
    def add_vc_statement(
        subject: str, *, timestamp: Optional[str] = None, graph_id: Optional[uuid.UUID] = None
    ) -> str: ...
    @staticmethod
    def add_storage_statement(
        data: str,
        stored_on: str,
        *,
        operated_by: Optional[str] = None,
        skip_proof: Optional[bool] = None,
        graph_id: Optional[uuid.UUID] = None,
    ) -> List[str]: ...
    @staticmethod
    def register_statement(statement_json: str) -> None:
        """Register a statement from JSON string to the default graph."""
        ...

    @staticmethod
    def register_statement_to_graph(statement_id: str, graph_id: str) -> None:
        """Associate an existing statement with a graph."""
        ...

    @staticmethod
    def create_model_signing_statement(
        collection_cid: str,
        blobs_dir: PathLike[str],
        model_signing_name: str,
        allow_symlinks: bool,
        ignore_paths: List[str],
        *,
        timestamp: Optional[str] = None,
        graph_id: Optional[uuid.UUID] = None,
    ) -> str: ...

# Stream module
class stream:
    @staticmethod
    def create(
        input_cids: List[str],
        operated_by: Optional[str] = None,
        executed_on: Optional[str] = None,
        timestamp: Optional[str] = None,
    ) -> Any:
        """creates a new computation stream"""
        ...

    @staticmethod
    def update(id: str, chunk: bytes) -> Any:
        """updates an existing computation stream with new data"""
        ...

    @staticmethod
    def finalize(
        id: str,
        static_output_cids: Optional[List[str]] = None,
        graph_id: Optional[uuid.UUID] = None,
    ) -> Any:
        """Finalizes the computation stream and creates the ComputationStatement and (optionally) the VC and VCStatement returns the CID of the computation statement"""
        ...
