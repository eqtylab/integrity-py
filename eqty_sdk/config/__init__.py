from pathlib import Path
from typing import Optional

from eqty_sdk._rust import init as _init
from eqty_sdk._rust import config as _config_module

# Re-export functions from Rust config module
__all__ = [
    "init",
    "blob_dir",
    "root_context",
    "get_store_all_blobs",
    "set_store_all_blobs",
    "get_cid_ignore_rules",
    "set_cid_ignore_rules",
    "get_integrity_service_url",
    "set_integrity_service_url",
    "get_generate_model_signing_signatures",
    "set_generate_model_signing_signatures",
    "get_app_dir",
    "get_blob_dir",
    "get_default_graph",
    "set_default_graph",
    "reset",
]


def init(app_dir: Optional[Path] = None) -> None:
    """Initialize the SDK."""
    _init(app_dir)


def blob_dir() -> Path:
    """Returns the blob directory as a Path object."""
    return Path(_config_module.get_blob_dir())


def root_context():
    """Returns the default graph (root context)."""
    return _config_module.get_default_graph()


# Re-export all config functions
get_store_all_blobs = _config_module.get_store_all_blobs
set_store_all_blobs = _config_module.set_store_all_blobs
get_cid_ignore_rules = _config_module.get_cid_ignore_rules
set_cid_ignore_rules = _config_module.set_cid_ignore_rules
get_integrity_service_url = _config_module.get_integrity_service_url
set_integrity_service_url = _config_module.set_integrity_service_url
get_generate_model_signing_signatures = _config_module.get_generate_model_signing_signatures
set_generate_model_signing_signatures = _config_module.set_generate_model_signing_signatures
get_app_dir = _config_module.get_app_dir
get_blob_dir = _config_module.get_blob_dir
get_default_graph = _config_module.get_default_graph
set_default_graph = _config_module.set_default_graph
reset = _config_module.reset
