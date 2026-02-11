from pathlib import Path
from typing import Optional

from eqty_sdk._rust import (
    get_cid_for_bytes as _get_cid_for_bytes,
    get_cid_for_path as _get_cid_for_path,
)


def get_cid_for_bytes(data: bytes, store: Optional[bool] = None) -> str:
    """Calculates and returns the CID for the provided bytes."""
    return _get_cid_for_bytes(data, store=store)


def get_cid_for_path(path: Path, store: Optional[bool] = None) -> str:
    """Resolves the provided path and reads the file or directory to calculate the cid."""
    return _get_cid_for_path(path, store=store)
