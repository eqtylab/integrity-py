# Re-export Cid from Rust module for backwards compatibility
from typing import TYPE_CHECKING

from eqty_sdk._rust import cid as _cid_module

if TYPE_CHECKING:
    from eqty_sdk._rust import Cid as Cid
else:
    Cid = _cid_module.Cid

__all__ = ["Cid"]
