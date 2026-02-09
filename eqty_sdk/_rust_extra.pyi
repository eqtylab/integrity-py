# Extra stubs layered on top of pyo3-stubgen output.
# This file is appended by scripts/merge_pyi.py.

from typing import Any, Dict, List, Optional, Tuple, Union
from os import PathLike


class Canon:
    RDFC1: Canon
    JSONJCS: Canon


class Cid:
    @property
    def cid(self) -> str: ...

    def __init__(self, cid: str) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...


class CidResult:
    @property
    def cid(self) -> str: ...

    @property
    def blob(self) -> bytes: ...


class DirCidResult:
    @property
    def collection(self) -> CidResult: ...

    @property
    def meta(self) -> CidResult: ...

    @property
    def file_hashes(self) -> List[Tuple[str, str]]: ...


class Entity:
    @property
    def uuid(self) -> str: ...

    def __init__(self, uuid: str) -> None: ...
    @staticmethod
    def generate() -> Entity: ...
    @staticmethod
    def from_uuid(uuid: str) -> Entity: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...


class Graph:
    @property
    def id(self) -> Any: ...

    @property
    def name(self) -> str: ...

    @property
    def parent(self) -> Optional[Any]: ...

    def __init__(self, id: Any, name: str) -> None: ...
    @staticmethod
    def from_parent(id: Any, name: str, graph: Graph) -> Graph: ...


class Manifest:
    @property
    def manifest_str(self) -> str: ...

    def __init__(self, manifest: str) -> None: ...
    @classmethod
    def from_statements(cls, statements: Any, include_context: bool = True) -> Manifest: ...
    def export(self, file: PathLike[str]) -> None: ...
    @classmethod
    def import_manifest(cls, manifest: Union[str, PathLike[str]]) -> None: ...
    @staticmethod
    def merge(a: str, b: str) -> str: ...
    def register(self) -> None: ...


class PySigner:
    @property
    def name(self) -> str: ...

    @property
    def did_key(self) -> str: ...


class cid:
    Cid: type[Cid]
    CidResult: type[CidResult]
    DirCidResult: type[DirCidResult]
    Canon: type[Canon]

    @staticmethod
    def compute_cid_for_directory(path: PathLike[str]) -> Any: ...
    @staticmethod
    def compute_cid_for_file(path: PathLike[str]) -> Any: ...
    @staticmethod
    def compute_cid_for_bytes(bytes: Any) -> str: ...


class entity:
    Entity: type[Entity]
    @staticmethod
    def create_entity(
        metadata_json: str,
        skip_proof: Optional[bool] = None,
        timestamp: Optional[str] = None,
        graph_id: Optional[Any] = None,
    ) -> Any: ...
    @staticmethod
    def create_entity_from_uuid(
        uuid: str,
        metadata_json: str,
        skip_proof: Optional[bool] = None,
        timestamp: Optional[str] = None,
        graph_id: Optional[Any] = None,
    ) -> Any: ...


class manifest:
    Manifest: type[Manifest]
    @staticmethod
    def generate(
        statements: List[Any], blobs_dir: PathLike[str], include_context: Optional[bool] = None
    ) -> str: ...
    @staticmethod
    def merge(a: str, b: str) -> str: ...
    @staticmethod
    def register(manifest: str, api_key: Optional[str] = None) -> None: ...


class signer:
    PySigner: type[PySigner]
    @staticmethod
    def create_new_signer(key_type: str, name: Optional[Any] = None) -> Any: ...
    @staticmethod
    def get_supported_signers() -> List[str]: ...


class config:
    Graph: type[Graph]
    @staticmethod
    def get_integrity_service_url() -> Optional[str]: ...
    @staticmethod
    def get_store_all_blobs() -> bool: ...
    @staticmethod
    def get_cid_ignore_rules() -> Any: ...
    @staticmethod
    def get_generate_model_signing_signatures() -> bool: ...
    @staticmethod
    def get_app_dir() -> Any: ...
    @staticmethod
    def get_blob_dir() -> Any: ...
    @staticmethod
    def get_default_graph() -> Graph: ...
    @staticmethod
    def set_integrity_service_url(url: str) -> None: ...
    @staticmethod
    def set_store_all_blobs(value: bool) -> None: ...
    @staticmethod
    def set_cid_ignore_rules(
        include_hidden_files: Optional[bool] = None,
        gitignore: Optional[bool] = None,
        include_symlinks: Optional[bool] = None,
    ) -> None: ...
    @staticmethod
    def set_generate_model_signing_signatures(enable: bool) -> None: ...
    @staticmethod
    def set_default_graph(graph: Any) -> None: ...
    @staticmethod
    def reset() -> None: ...


class statements:
    @staticmethod
    def retrieve_graph(graph_ids: List[Any]) -> Any: ...

