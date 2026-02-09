import logging
import os
from pathlib import Path
from typing import List, Optional

from eqty_sdk import config
from eqty_sdk._rust import (
    statements as eqty_core_statements,
)

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_metadata_statement(
    subject_cid: str,
    metadata: str,
    skip_proof: Optional[bool] = None,
) -> List[str]:
    """Add a metadata registration statement attached to the subject_cid."""
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    logger.debug(f"Creating metadata statement. {metadata}")
    (statement_id, metadata_cid) = eqty_core_statements.create_metadata_statement(
        subject_cid, metadata, timestamp=timestamp
    )

    statement_ids = [statement_id]

    metadata_jcs_file = config.blob_dir()
    metadata_file = Path(metadata_jcs_file, metadata_cid[len("urn:cid:") :])
    with open(metadata_file, "w") as f:
        f.write(metadata)

    vc_id = add_vc_statement(statement_id, timestamp, skip_proof)
    if vc_id:
        statement_ids.append(vc_id)

    return statement_ids
