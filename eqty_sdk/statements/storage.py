import logging
import os
from typing import List, Optional

from eqty_sdk._rust import statements as eqty_core_statements

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_storage_statement(
    data: str,
    stored_on: str,
    operated_by: Optional[str],
    skip_proof: Optional[bool] = None,
) -> List[str]:
    """Creates a new storage statement."""
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    statement_id = eqty_core_statements.create_storage_statement(
        data, stored_on, operated_by, timestamp
    )

    statement_ids = [statement_id]

    vc_id = add_vc_statement(statement_id, timestamp, skip_proof)
    if vc_id:
        statement_ids.append(vc_id)

    return statement_ids
