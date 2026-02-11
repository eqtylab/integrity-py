from typing import TYPE_CHECKING, Any, Protocol, runtime_checkable

from eqty_sdk._rust import (
    Did as _Did,
    cid as _cid,
    entity as _entity,
    manifest as _manifest,
    signer as _signer,
)

from .declaration import Declaration

if TYPE_CHECKING:

    @runtime_checkable
    class Cid(Protocol):
        cid: str

    @runtime_checkable
    class Entity(Protocol):
        uuid: str

    @runtime_checkable
    class Manifest(Protocol):
        manifest_str: str

    @runtime_checkable
    class Signer(Protocol):
        name: str
        did_key: str

    class Did(Protocol):
        ctx: Any
        statement_ids: list[str]

        @staticmethod
        def from_signer(signer: Signer, **kwargs: Any) -> "Did": ...

        @staticmethod
        def from_did_string(did: str, **kwargs: Any) -> "Did": ...

        @staticmethod
        def with_context(ctx: Any) -> Any: ...

    class SIGNER_ALGORITHMS(Protocol):
        pass
else:
    Cid = _cid.Cid
    Entity = _entity.Entity
    Manifest = _manifest.Manifest
    Signer = _signer.Signer
    Did = _Did
    SIGNER_ALGORITHMS = _signer.SIGNER_ALGORITHMS

set_active_signer = _signer.set_active_signer

__all__ = [
    "Cid",
    "Declaration",
    "Did",
    "Signer",
    "set_active_signer",
    "SIGNER_ALGORITHMS",
    "Entity",
    "Manifest",
]
