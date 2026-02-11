from pathlib import Path
from typing import Optional

from eqty_sdk._rust import (
    Config as _Config,
    init as _init,
)

__all__ = ["init"]


def init(app_dir: Optional[Path] = None) -> _Config:
    """Initialize the SDK and return the config instance."""
    return _init(app_dir)
