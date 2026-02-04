import logging
import os
from typing import Optional

from eqty_sdk._rust import (
    Graph as Context,
    statements as eqty_core_statements,
)

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_entity_statement(
    entity: str,
    skip_proof: Optional[bool],
    ctx: Optional[Context] = None,
) -> None:
    """Add an entity statement."""
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    statement_id = eqty_core_statements.create_entity_statement(
        [entity], timestamp, graph_id=ctx.id if ctx else None
    )

    add_vc_statement(statement_id, timestamp, skip_proof)
