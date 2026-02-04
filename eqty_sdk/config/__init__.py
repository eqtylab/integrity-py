from pathlib import Path
from typing import Optional

from eqty_sdk._rust import Config, init as _init

# Re-export Config class
__all__ = ["Config", "init", "blob_dir", "root_context"]

# Store the config instance after init
_config: Optional[Config] = None


def init(app_dir: Optional[Path] = None) -> Config:
    """Initialize the SDK and return the Config instance."""
    global _config
    _config = _init(app_dir)
    return _config


def get_config() -> Config:
    """Get the current config instance, raising if not initialized."""
    if _config is None:
        raise RuntimeError("Config not initialized. Call init() first.")
    return _config


def blob_dir() -> Path:
    """Returns the blob directory as a Path object."""
    return Path(get_config().get_blob_dir())


def root_context():
    """Returns the default graph (root context)."""
    return get_config().get_default_graph()
