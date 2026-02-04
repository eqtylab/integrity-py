from pathlib import Path

from eqty_sdk._rust import config as _rust_config

from .cid_ignore import CidIgnore

# Re-export Rust config functions
init = _rust_config.init
reset = _rust_config.reset

# Setters
set_integrity_service_url = _rust_config.set_integrity_service_url
set_hashing_config = _rust_config.set_hashing_config
set_cid_ignore_rules = _rust_config.set_cid_ignore_rules
set_generate_model_signing_signatures = _rust_config.set_generate_model_signing_signatures
set_default_graph = _rust_config.set_default_graph
set_store_all_blobs = _rust_config.set_store_all_blobs

# Getters
get_integrity_service_url = _rust_config.get_integrity_service_url
get_store_all_blobs = _rust_config.get_store_all_blobs
get_cid_ignore_rules = _rust_config.get_cid_ignore_rules
get_generate_model_signing_signatures = _rust_config.get_generate_model_signing_signatures
get_app_dir = _rust_config.get_app_dir
get_blob_dir = _rust_config.get_blob_dir


def blob_dir() -> Path:
    """Returns the blob directory as a Path object."""
    return Path(get_blob_dir())


def cid_ignore() -> CidIgnore:
    """Returns the current CID ignore settings."""
    hidden, gitignore, symlinks = get_cid_ignore_rules()
    return CidIgnore(
        include_hidden_files=hidden,
        gitignore=gitignore,
        include_symlinks=symlinks,
    )


__all__ = [
    "CidIgnore",
    "init",
    "reset",
    "set_integrity_service_url",
    "set_hashing_config",
    "set_cid_ignore_rules",
    "set_generate_model_signing_signatures",
    "set_default_graph",
    "set_store_all_blobs",
    "get_integrity_service_url",
    "get_store_all_blobs",
    "get_cid_ignore_rules",
    "get_generate_model_signing_signatures",
    "get_app_dir",
    "get_blob_dir",
    "blob_dir",
    "cid_ignore",
]
