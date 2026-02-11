import logging
import os
from typing import Optional
from uuid import uuid4

from eqty_sdk._rust import (
    statements as eqty_core_statements,
)

logger = logging.getLogger("eqty.sdk.statements")


def add_vc_statement(
    subject: str,
    timestamp: Optional[str],
    skip_proof: Optional[bool],
) -> Optional[str]:
    """Creates a VC statement for the prvided subject ONLY if skip_proof is false.
    If a VC Statement is created, the statement id is returned.
    """
    if skip_proof or (skip_proof is None and os.getenv("EQTY_SKIP_PROOF", "").lower() == "true"):
        logger.info("Skipping issuing of VC")
        return None

    statement_id = eqty_core_statements.create_vc_statement(
        subject, timestamp=timestamp, graph_id=uuid4()
    )

    raise RuntimeError("add_vc_statement is not implemented yet")
    # return cast(str, statement_id)
