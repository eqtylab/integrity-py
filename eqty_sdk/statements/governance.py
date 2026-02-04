import logging
import os
from typing import Optional

from eqty_sdk._rust import (
    Graph as Context,
    statements as eqty_core_statements,
)

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_governance_statement(
    subject: str,
    document: str,
    skip_proof: Optional[bool] = None,
    ctx: Optional[Context] = None,
) -> None:
    """Creates a new governance statement."""
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    statement_id = eqty_core_statements.create_governance_statement(
        subject, document, timestamp, graph_id=ctx.id if ctx else None
    )

    add_vc_statement(statement_id, timestamp, skip_proof)
