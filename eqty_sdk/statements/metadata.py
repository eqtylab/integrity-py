import logging
import os
from pathlib import Path
from typing import Optional

from eqty_sdk._rust import (
    Graph as Context,
    statements as eqty_core_statements,
)
from eqty_sdk.config import Config

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_metadata_statement(
    subject_cid: str,
    metadata: str,
    skip_proof: Optional[bool] = None,
    ctx: Optional[Context] = None,
) -> None:
    """Add a metadata registration statement attached to the subject_cid."""
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    logger.debug(f"Creating metadata statement. {metadata}")
    (statement_id, metadata_cid) = eqty_core_statements.create_metadata_statement(
        subject_cid, metadata, timestamp=timestamp, graph_id=ctx.id if ctx else None
    )

    metadata_jcs_file = Config().blob_dir()
    metadata_file = Path(metadata_jcs_file, metadata_cid[len("urn:cid:") :])
    with open(metadata_file, "w") as f:
        f.write(metadata)

    add_vc_statement(statement_id, timestamp, skip_proof)
