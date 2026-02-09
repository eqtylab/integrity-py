import logging
import shutil
from pathlib import Path
from typing import Optional, cast

from eqty_sdk._rust import cid as eqty_core_cid

from . import config

logger = logging.getLogger("eqty.sdk")


def __get_store_flag__(store: Optional[bool]) -> bool:
    """Checks the 'store' argument and the Config store_all_blobs setting."""
    # if `store` is explicitly set, use that value, otherwise use the global config setting
    if store is False or store is True:
        return store
    else:
        return config.get_store_all_blobs()


def get_cid_for_bytes(data: bytes, store: Optional[bool] = None) -> str:
    """Calculates and returns the CID for the provided bytes."""
    store_flag = __get_store_flag__(store)
    cid = cast(
        str,
        eqty_core_cid.compute_cid_for_bytes(data),
    )

    if store_flag:
        file = config.blob_dir() / cid
        with open(file, "wb") as f:
            f.write(data)

    return cid


def get_cid_for_path(path: Path, store: Optional[bool] = None) -> str:
    """Resolves the provide path and reads the file or directory to calculate the cid."""
    store_flag = __get_store_flag__(store)

    if path.is_file():
        file_cid_results = eqty_core_cid.compute_cid_for_file(path)
        cid = file_cid_results.cid
        if store_flag:
            storage_dir = config.blob_dir() / cid
            shutil.copy2(path, storage_dir)

        return cast(str, cid)
    elif path.is_dir():
        dir_cid_results = eqty_core_cid.compute_cid_for_directory(path)
        cid = dir_cid_results.collection.cid
        # Always store iroh collections
        logger.info(f"Saving iroh collection {dir_cid_results.collection.cid}")
        collection_file = config.blob_dir() / dir_cid_results.collection.cid
        collection_file.write_bytes(dir_cid_results.collection.blob)
        meta_file = config.blob_dir() / dir_cid_results.meta.cid
        meta_file.write_bytes(dir_cid_results.meta.blob)

        if store_flag:
            logger.info("Saving iroh collection blobs")
            for blob in dir_cid_results.file_hashes:
                src = path.joinpath(blob[0])
                dst = config.blob_dir() / blob[1]
                logger.debug(f"copying iroh blob from {src} to {dst}")
                shutil.copy(src, dst)

        return cast(str, cid)
    else:
        msg = f"The provided path {path} was not found"
        logger.error(msg)
        raise RuntimeError(msg)
