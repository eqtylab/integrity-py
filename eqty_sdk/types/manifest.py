"""Manifest type re-exported from Rust implementation."""

from typing import TYPE_CHECKING

from eqty_sdk._rust import manifest as _manifest_module

if TYPE_CHECKING:
    from eqty_sdk._rust import Manifest as Manifest
else:
    Manifest = _manifest_module.Manifest

__all__ = ["Manifest"]
