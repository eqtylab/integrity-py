import logging
import os
import sqlite3
from pathlib import Path

from eqty_sdk import SIGNER_ALGORITHMS, Signer, config, set_active_signer
from eqty_sdk._rust import Config

test_dir = Path("tmp")
CONFIG = None
logger = logging.getLogger("unit_tests_root")


def setup_sdk() -> Config:
    """Initializes the SDK and sets a known signer."""
    _configure_debug_logging()
    global CONFIG
    if not CONFIG:
        CONFIG = config.init(test_dir)
        signer = Signer.from_private_key(
            algorithm=SIGNER_ALGORITHMS.ED25519,
            private_key="eHb22WNFvUXihogn8fubQjW7hHEqwY3fEKt745V4xXg=",
        )
        set_active_signer(signer)
        return CONFIG
    else:
        return CONFIG


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
