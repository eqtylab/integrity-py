import logging
import os
import shutil
import sqlite3
from pathlib import Path

from eqty_sdk import SIGNER_ALGORITHMS, Signer, config, set_active_signer

test_dir = Path("tmp")
CONFIG_INITIALIZED = False
logger = logging.getLogger("unit_tests_root")


def setup_sdk() -> None:
    """Initializes the SDK and sets a known signer."""
    _configure_debug_logging()
    global CONFIG_INITIALIZED
    if not CONFIG_INITIALIZED:
        config.init(test_dir)
        signer = Signer.from_private_key(
            algorithm=SIGNER_ALGORITHMS.ED25519,
            private_key="eHb22WNFvUXihogn8fubQjW7hHEqwY3fEKt745V4xXg=",
        )
        set_active_signer(signer)
        CONFIG_INITIALIZED = True


def get_config_dir() -> Path:
    """Returns the directory that the eqty_sdk was initialized to."""
    return test_dir


def get_statement_count_by_type(statement_type: str) -> int | None:
    """Returns the number of statements in the database of the specified type."""
    db_file = Path(test_dir).joinpath("graphs.db")

    logger.info("getting statement count by type %s from %s", statement_type, db_file)
    with sqlite3.connect(db_file) as conn:
        cursor = conn.cursor()
        if statement_type == "CredentialRegistration":
            cursor.execute("SELECT COUNT(*) FROM credential_statements")
        elif statement_type == "CredentialSigstoreBundleRegistration":
            cursor.execute("SELECT COUNT(*) FROM sigstore_statements")
        elif statement_type == "CredentialDsseRegistration":
            cursor.execute("SELECT COUNT(*) FROM dsse_statements")
        else:
            raise ValueError(f"Unsupported statement type for count: {statement_type}")
        result = cursor.fetchone()
        return int(result[0]) if result else 0


def reset_statement_db() -> None:
    """Deletes all records from the 'statements' table in sqlite."""
    db_file = Path(test_dir).joinpath("graphs.db")

    try:
        with sqlite3.connect(db_file) as conn:
            cursor = conn.cursor()
            cursor.execute("DELETE FROM credential_statements")
            cursor.execute("DELETE FROM sigstore_statements")
            cursor.execute("DELETE FROM dsse_statements")
            conn.commit()
    except Exception as e:
        print("unable to delete credential tables", e)


def clean_blobs() -> None:
    """Deletes the blobs directory."""
    blob_dir = test_dir.joinpath("blobs")
    if not os.path.exists(blob_dir):
        return

    logger.info("Deleting blobs dir %s", blob_dir)
    shutil.rmtree(blob_dir)


def _configure_debug_logging():
    """Enables debug logging if env var 'EQTY_PY_TEST_LOGGING' is True."""
    logging_enabled = os.getenv("EQTY_PY_TEST_LOGGING", "").lower() in ("true", "1", "yes", "on")
    if logging_enabled:
        logging.basicConfig(
            level=logging.DEBUG,
            format="%(levelname)s - %(name)s %(funcName)s: %(message)s",
            handlers=[logging.StreamHandler()],
        )

        sdk_logger = logging.getLogger("eqty.sdk")
        sdk_logger.setLevel(logging.DEBUG)
