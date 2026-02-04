import logging
import os
from typing import Optional

from eqty_sdk._rust import (
    Graph as Context,
    statements as eqty_core_statements,
)

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_storage_statement(
    data: str,
    stored_on: str,
    operated_by: Optional[str],
    skip_proof: Optional[bool] = None,
    ctx: Optional[Context] = None,
) -> None:
    """Creates a new storage statement."""
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    statement_id = eqty_core_statements.create_storage_statement(
        data, stored_on, operated_by, timestamp, graph_id=ctx.id if ctx else None
    )

    add_vc_statement(statement_id, timestamp, skip_proof)
