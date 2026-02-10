from typing import TYPE_CHECKING, Protocol, runtime_checkable

from eqty_sdk._rust import cid as _cid
from eqty_sdk._rust import entity as _entity
from eqty_sdk._rust import manifest as _manifest
from eqty_sdk._rust import signer as _signer

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

    class SIGNER_ALGORITHMS(Protocol):
        pass
else:
    Cid = _cid.Cid
    Entity = _entity.Entity
    Manifest = _manifest.Manifest
    Signer = _signer.Signer
    SIGNER_ALGORITHMS = _signer.SIGNER_ALGORITHMS

set_active_signer = _signer.set_active_signer

from .declaration import Declaration
from .did import Did

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
