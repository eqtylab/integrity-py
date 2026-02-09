import logging
import os
from typing import List, Optional

from eqty_sdk._rust import statements as eqty_core_statements

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_entity_statement(
    entity: str,
    skip_proof: Optional[bool],
) -> List[str]:
    """Add an entity statement."""
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    statement_id = eqty_core_statements.create_entity_statement([entity], timestamp)

    statement_ids = [statement_id]

    vc_id = add_vc_statement(statement_id, timestamp, skip_proof)
    if vc_id:
        statement_ids.append(vc_id)

    return statement_ids
