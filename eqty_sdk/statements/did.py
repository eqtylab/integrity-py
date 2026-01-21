import logging
import os
from typing import List, Optional

from eqty_sdk._rust import statements as eqty_core_statements

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_did_statement(
    did: str,
    skip_proof: Optional[bool] = None,
) -> List[str]:
    """Creates a new DID statement."""
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    statement_id = eqty_core_statements.create_did_statement(did, timestamp)

    statement_ids = [statement_id]

    vc_id = add_vc_statement(statement_id, timestamp, skip_proof)
    if vc_id:
        statement_ids.append(vc_id)

    return statement_ids
