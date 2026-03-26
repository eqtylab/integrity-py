"""Type stubs for the eqty_sdk._rust module."""

import eqty_sdk
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Union
import uuid
from os import PathLike

def init(
    default_context: Optional[Context] = None, *, custom_dir: Optional[PathLike[str]] = None
) -> Config:
    """Initializes the sdk config. Must be called before setting individual config values"""
    ...

def get_cid_for_bytes(data: bytes, _store: Optional[bool] = None) -> CID:
    """Calculates and returns the CID for the provided bytes."""
    ...

def get_cid_for_json(json: str, _store: Optional[bool] = None) -> CID:
    """Calculates and returns the JCS CID for the provided JSON string."""
    ...

def get_cid_for_path(path: PathLike[str], _store: Optional[bool] = None) -> CID:
    """Resolves the provided path and reads the file or directory to calculate the CID. The path is saved to the blob store if the store flag is set"""
    ...

def purge_statement_store() -> None:
    """Purges all statemetns from the store."""
    ...

def purge_blob_store() -> None:
    """Purges all blobs from the blob store."""
    ...

class CID:
    """A simple wrapper around a content identifier (CID) string.  Provides a typed wrapper for CID strings with property access and string conversion."""
    @property
    def cid(self) -> str:
        """The formatted CID string."""
        ...

    def __init__(self, cid: str) -> None:
        """Creates a new CID, ensuring it is prefixed with `urn:cid:`."""
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __len__(self) -> int: ...
    def __getitem__(self, index: Any) -> str: ...
    def startswith(self, prefix: str) -> bool: ...
    def __eq__(self, other: Any) -> bool: ...

class Config:
    """Global application config containing configuration and state.  The config stores application-wide settings including storage directories, hashing preferences, and file filtering rules."""
    def set_hashing_config(
        self, multithread: Optional[bool] = None, memory_map: Optional[bool] = None
    ) -> Config:
        """Updates hashing behavior used when computing CIDs.  - `multithread`: Enable multithreaded hashing. - `memory_map`: Enable memory-mapped file reads where supported.  Returns the updated config instance."""
        ...

    def set_cid_ignore_rules(
        self,
        include_hidden_files: Optional[bool] = None,
        gitignore: Optional[bool] = None,
        include_symlinks: Optional[bool] = None,
    ) -> Config:
        """Updates the directory ignore rules used when computing CIDs.  - `include_hidden_files`: Include hidden files in directory hashing. - `gitignore`: Respect `.gitignore` rules while hashing directories. - `include_symlinks`: Include symlinks in directory hashing.  Returns the updated config instance."""
        ...

    def set_store_all_blobs(self, value: bool) -> Config:
        """Sets whether blobs should be persisted automatically when computing CIDs.  Returns the updated config instance."""
        ...

    def get_default_context(self) -> Context:
        """Returns the default context used when no context is supplied explicitly."""
        ...

class Context:
    """A structure for organizing related statements hierarchically in the database.  Graph context groups statements together with optional parent-child relationships, enabling organizational structure for lineage graphs."""
    @property
    def id(self) -> uuid.UUID:
        """Unique identifier"""
        ...

    @property
    def name(self) -> str:
        """Human-readable name"""
        ...

    @property
    def parent(self) -> Optional[uuid.UUID]:
        """Optional parent ID for hierarchical organization"""
        ...

    @staticmethod
    def new(name: str) -> Context:
        """Creates a new context with the given name.  If the global config is initialized, the context is persisted to sqlite."""
        ...

    @staticmethod
    def with_parent(parent: Context) -> ContextFactory:
        """Returns a factory that creates contexts with the provided parent."""
        ...

    @staticmethod
    def from_uuid(project_id: uuid.UUID) -> Context:
        """Creates a new context with the given uuid."""
        ...

    def register(self, service: Service) -> None:
        """Registers this context, its ancestors, statements, and blobs with a service."""
        ...

    def delete_tree(self) -> None: ...
    def delete(self) -> None: ...
    def export(self, path: PathLike[str]) -> None:
        """Exports this context's statements and blobs to a manifest JSON file."""
        ...

    def import_manifest(self, path: PathLike[str]) -> None:
        """Imports the statements and blobs from a manifest file to this context."""
        ...

    def __str__(self) -> str: ...

class ContextFactory:
    """Factory for creating contexts with an optional parent."""
    def new(self, name: str) -> Context:
        """Creates a new context using the factory's parent if set."""
        ...

class DID:
    """DID object"""
    @property
    def did(self) -> str:
        """DID string used for registration."""
        ...

    def __init__(self, did: str, **kwargs: Any) -> None: ...
    @staticmethod
    def from_signer(signer: Signer, **kwargs: Any) -> DID: ...
    @staticmethod
    def from_did_string(did: str, **kwargs: Any) -> DID: ...

class Entity:
    """Represents an unhashed entity with a UUID identifier.  Entities are used to represent objects that don't have a content-based identifier (CID) but need a unique identifier for tracking purposes."""
    @property
    def uuid(self) -> str: ...
    def __init__(self, uuid: str) -> None:
        """Create a new Entity with the given UUID string."""
        ...

    @staticmethod
    def generate() -> Entity:
        """Create a new Entity with a randomly generated UUID."""
        ...

    @staticmethod
    def from_uuid(uuid: str) -> Entity:
        """Create an Entity from a UUID string."""
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class PyAssociationType:
    Certifies: PyAssociationType
    Includes: PyAssociationType
    IsInstanceOf: PyAssociationType

class SIGNER_ALGORITHMS:
    """Supported signer algorithm identifiers."""

    ED25519: SIGNER_ALGORITHMS
    SECP256K1: SIGNER_ALGORITHMS
    SECP256R1: SIGNER_ALGORITHMS

class Service:
    """Service for connecting to the Integrity Service API."""
    @property
    def base_path(self) -> str:
        """Base URL path for the API (e.g., <https://api.example.com>)."""
        ...

    @staticmethod
    def new(url: str, api_key: Optional[str] = None) -> Service:
        """Creates a service client using the provided URL and API key."""
        ...

class Signer:
    """Python-exposed signer information.  Contains the name and DID key of a cryptographic signer."""
    @property
    def name(self) -> str:
        """Returns the human-readable name of the signer.  # Returns * `&str` - The signer's name"""
        ...

    @property
    def did_key(self) -> str:
        """Returns the DID key of the signer.  # Returns * `&str` - The signer's DID key string"""
        ...

    def __init__(self, name: str, did_key: str) -> None: ...
    @staticmethod
    def new(
        algorithm: Optional[SIGNER_ALGORITHMS] = None,
        name: Optional[str] = None,
        _load_if_exists: Optional[bool] = None,
    ) -> Signer: ...
    @staticmethod
    def vcomp_notary(
        url: Optional[str] = None,
        name: Optional[str] = None,
        _load_if_exists: Optional[bool] = None,
    ) -> Signer:
        """Creates a VComp notary signer and persists it to disk.  If `name` is provided, the signer is stored under that name. When `_load_if_exists=True`, an existing signer with the same name is loaded instead of creating a new remote signer configuration."""
        ...

    @staticmethod
    def auth_service(
        url: str, name: Optional[str] = None, _load_if_exists: Optional[bool] = None
    ) -> Signer:
        """Creates an Auth Service signer and persists it to disk.  Requires the `EQTY_API_KEY` environment variable to be set. If `name` is provided, the signer is stored under that name. When `_load_if_exists=True`, an existing signer with the same name is loaded instead of creating a new remote signer configuration."""
        ...

    @staticmethod
    def from_private_key(
        algorithm: SIGNER_ALGORITHMS,
        private_key: str,
        name: Optional[str] = None,
        _load_if_exists: Optional[bool] = None,
    ) -> Signer: ...

class UUID:
    """A simple wrapper around a UUID string.  Provides a typed wrapper for UUID strings with property access and string conversion."""
    @property
    def uuid(self) -> str:
        """The formatted UUID string."""
        ...

    def __init__(self, uuid: str) -> None:
        """Creates a new UUID, ensuring it is prefixed with `urn:uuid:`."""
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __len__(self) -> int: ...
    def __getitem__(self, index: Any) -> str: ...
    def startswith(self, prefix: str) -> bool: ...
    def __eq__(self, other: Any) -> bool: ...

_CID = CID
_Config = Config
_Context = Context
_ContextFactory = ContextFactory
_DID = DID
_Entity = Entity
_PyAssociationType = PyAssociationType
_SIGNER_ALGORITHMS = SIGNER_ALGORITHMS
_Service = Service
_Signer = Signer
_UUID = UUID

# Entity module
class entity:
    Entity: type[Entity]

    @staticmethod
    def create_entity(
        metadata_json: str,
        _skip_proof: Optional[bool] = None,
        timestamp: Optional[str] = None,
        context: Optional[_Context] = None,
    ) -> Tuple[_Entity, Any]:
        """# Arguments * `metadata_json` - JSON string containing metadata to associate with the entity * `_skip_proof` - If true, skip creating a VC statement * `timestamp` - Optional timestamp for statements * `context` - Optional Context to register statements to  # Returns Tuple of (Entity, list of statement IDs)"""
        ...

    @staticmethod
    def create_entity_from_uuid(
        uuid: str,
        metadata_json: str,
        _skip_proof: Optional[bool] = None,
        timestamp: Optional[str] = None,
        context: Optional[_Context] = None,
    ) -> Tuple[_Entity, Any]:
        """# Arguments * `uuid` - UUID string for the entity * `metadata_json` - JSON string containing metadata to associate with the entity * `_skip_proof` - If true, skip creating a VC statement * `timestamp` - Optional timestamp for statements * `context` - Optional Context to register statements to  # Returns Tuple of (Entity, list of statement IDs)"""
        ...

# Signer module
class signer:
    Signer: type[Signer]
    SIGNER_ALGORITHMS: type[SIGNER_ALGORITHMS]

    @staticmethod
    def set_active_signer(signer: _Signer) -> None:
        """Sets the active signer from a signer instance.  # Arguments * `signer` - Signer instance"""
        ...

    @staticmethod
    def get_active_signer_did_key() -> str:
        """Returns the DID key of the currently active signer."""
        ...

# Statements module
class statements:
    PyAssociationType: type[PyAssociationType]

    @staticmethod
    def add_association_statement(
        subject: str,
        association: List[str],
        association_type: _PyAssociationType,
        *,
        _skip_proof: Optional[bool] = None,
        context: Optional[_Context] = None,
    ) -> List[_CID]: ...
    @staticmethod
    def add_computation_statement(
        inputs: List[_CID],
        outputs: List[_CID],
        computation: Optional[_CID] = None,
        *,
        operated_by: Optional[str] = None,
        executed_on: Optional[str] = None,
        _skip_proof: Optional[bool] = None,
        context: Optional[_Context] = None,
    ) -> List[_CID]: ...
    @staticmethod
    def add_data_statement(
        data: List[_CID], *, _skip_proof: Optional[bool] = None, context: Optional[_Context] = None
    ) -> List[_CID]: ...
    @staticmethod
    def add_did_statement(
        did: str, *, _skip_proof: Optional[bool] = None, context: Optional[_Context] = None
    ) -> List[_CID]: ...
    @staticmethod
    def add_entity_statement(
        entity: str, *, _skip_proof: Optional[bool] = None, context: Optional[_Context] = None
    ) -> List[_CID]: ...
    @staticmethod
    def add_governance_statement(
        subject: str,
        document: str,
        *,
        _skip_proof: Optional[bool] = None,
        context: Optional[_Context] = None,
    ) -> List[_CID]: ...
    @staticmethod
    def add_metadata_statement(
        subject: str,
        metadata: str,
        *,
        _skip_proof: Optional[bool] = None,
        context: Optional[_Context] = None,
    ) -> List[_CID]: ...
    @staticmethod
    def add_vc_statement(
        subject: str, *, timestamp: Optional[str] = None, context: Optional[_Context] = None
    ) -> _CID: ...
    @staticmethod
    def add_storage_statement(
        data: str,
        stored_on: str,
        *,
        operated_by: Optional[str] = None,
        _skip_proof: Optional[bool] = None,
        context: Optional[_Context] = None,
    ) -> List[_CID]:
        """Adds a storage statement linking data to a storage location."""
        ...

    @staticmethod
    def register_statement(statement_json: str) -> None:
        """Register a statement from JSON string to the default context."""
        ...

    @staticmethod
    def add_model_signing_statement(
        collection_cid: str, model_signing_name: str, *, context: Optional[_Context] = None
    ) -> _CID: ...

# Stream module
class stream:
    @staticmethod
    def create(
        input_cids: List[_CID],
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
        id: str, static_output_cids: Optional[List[str]] = None, context: Optional[_Context] = None
    ) -> Any:
        """Finalizes the computation stream and creates the ComputationStatement and (optionally) the VC and VCStatement returns the CID of the computation statement"""
        ...
