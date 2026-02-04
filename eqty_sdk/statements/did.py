import logging
import os
from typing import Optional

from eqty_sdk._rust import (
    Graph as Context,
    statements as eqty_core_statements,
)

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_did_statement(
    did: str,
    skip_proof: Optional[bool] = None,
    ctx: Optional[Context] = None,
) -> None:
    """Creates a new DID statement."""
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    statement_id = eqty_core_statements.create_did_statement(
        did, timestamp=timestamp, graph_id=ctx.id if ctx else None
    )

    add_vc_statement(statement_id, timestamp, skip_proof)
