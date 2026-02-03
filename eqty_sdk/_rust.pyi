"""Type stubs for the eqty_sdk._rust module."""
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Union
from os import PathLike

class Canon:
    """Canonicalization options."""
    RDFC1: Canon
    JSONJCS: Canon


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


# Context module
class context:
    @staticmethod
    def init(app_dir: PathLike[str]) -> None:
        """Initializes the sdk context. Must be called before setting individual context values"""
        ...

    @staticmethod
    def reset() -> None:
        """Resets the global context, allowing it to be reinitialized with a new app directory"""
        ...

    @staticmethod
    def set_integrity_service_url(url: str) -> None:
        """Set Integrity Service Url."""
        ...

    @staticmethod
    def set_hashing_config(multithread: Optional[bool] = None, memory_map: Optional[bool] = None) -> None:
        """Set Hashing Config."""
        ...

    @staticmethod
    def set_cid_ignore_rules(include_hidden_files: Optional[bool] = None, gitignore: Optional[bool] = None, include_symlinks: Optional[bool] = None) -> None:
        """Set Cid Ignore Rules."""
        ...

    @staticmethod
    def set_generate_model_signing_signatures(enable: bool) -> None:
        """Set Generate Model Signing Signatures."""
        ...

    @staticmethod
    def set_default_graph(graph: Any) -> None:
        """Set Default Graph."""
        ...


# Manifest module
class manifest:
    @staticmethod
    def generate(statements: List[Any], blobs_dir: PathLike[str], include_context: Optional[bool] = None) -> str:
        """ # Arguments * `py` - Python interpreter reference * `graphs` - Python list of graph dictionaries, each containing 'id', 'name', 'parent', and 'statements' * `blobs_dir` - Path to directory containing blob files referenced by statements * `include_context` - Whether to include context information in the manifest (default: false)  # Returns * `PyResult<String>` - JSON string representation of the manifest, or error on failure"""
        ...

    @staticmethod
    def import_manifest(manifest: str, graph_id: Optional[Any] = None) -> Any:
        """Imports a manifest and returns the decoded blobs that must be saved"""
        ...

    @staticmethod
    def merge(a: str, b: str) -> str:
        """Merges the manifests `a` and `b` and returns the merged manifest."""
        ...

    @staticmethod
    def register(manifest: str, api_key: Optional[str] = None) -> None:
        """Register the manfiest with integrity platform"""
        ...


# Cid module
class cid:
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
    def set_active_signer(name: str) -> None:
        """Sets the active signer by name.  # Arguments * `name` - Name of the signer to make active"""
        ...

    @staticmethod
    def get_active_signer_did_key() -> str:
        """Get the active signers Did Key"""
        ...

    @staticmethod
    def get_signer_type(name: str) -> str:
        """Get signer type string ('vcomp_notary', 'yubihsm2', etc) by name.  # Arguments * `name` - Name of the signer to retrieve"""
        ...

    @staticmethod
    def get_signer_statements(name: str) -> List[str]:
        """Retrieves the statements associated with a signer if any.  # Arguments * `name` - Name of the signer to retrieve statements from"""
        ...

    @staticmethod
    def get_signer_blobs(name: str) -> Any:
        """Retrieves the blobs associated with a signer if any.  # Arguments * `name` - Name of the signer to retrieve blobs from"""
        ...


# Statements module
class statements:
    @staticmethod
    def retrieve_graph(graph_ids: List[str]) -> Any:
        """Retrieve graphs for multiple graph IDs.  Args: graph_ids: List of graph ID strings to retrieve graphs for  Returns: List of graph objects with their statements"""
        ...

    @staticmethod
    def create_metadata_statement(subject: str, metadata: str, timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> Any:
        """Creates a metadata statement and returns the ID of the statement and the CID of the metadata Json"""
        ...

    @staticmethod
    def create_data_statement(data: List[str], *, timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> str:
        """Create Data Statement."""
        ...

    @staticmethod
    def create_governance_statement(subject: str, document: str, timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> str:
        """Create Governance Statement."""
        ...

    @staticmethod
    def create_computation_statement(inputs: List[str], outputs: List[str], computation: Optional[str] = None, operated_by: Optional[str] = None, executed_on: Optional[str] = None, timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> str:
        """Create Computation Statement."""
        ...

    @staticmethod
    def create_model_signing_statement(collection_cid: str, blobs_dir: PathLike[str], model_signing_name: str, allow_symlinks: bool, ignore_paths: List[str], timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> str:
        """Create Model Signing Statement."""
        ...

    @staticmethod
    def create_association_statement(subject: str, association: str, timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> str:
        """Create Association Statement."""
        ...

    @staticmethod
    def create_entity_statement(entity: List[str], timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> str:
        """Create Entity Statement."""
        ...

    @staticmethod
    def create_did_statement(did: str, timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> str:
        """Create Did Statement."""
        ...

    @staticmethod
    def create_storage_statement(data: str, stored_on: str, operated_by: Optional[str] = None, timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> str:
        """Creates a storage statement."""
        ...

    @staticmethod
    def create_vc_statement(subject: str, timestamp: Optional[str] = None, graph_id: Optional[Any] = None) -> str:
        """Create Vc Statement."""
        ...

