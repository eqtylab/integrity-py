import logging
import os
from typing import List, Optional

from eqty_sdk._rust import statements as eqty_core_statements

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_association_statement(
    subject: str,
    association: str,
    skip_proof: Optional[bool] = None,
) -> List[str]:
    """Creates a new association statement.

    Args:
        subject: The subject of the association (CID, URN, or DID)
        association: The association identifier (CID, URN, or DID)
        skip_proof: Whether to skip proof generation (optional)

    Returns:
        List[str]: The list of IDs of the created statements.

    """
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    statement_id = eqty_core_statements.create_association_statement(
        subject, association, timestamp
    )

    statement_ids = [statement_id]

    vc_id = add_vc_statement(statement_id, timestamp, skip_proof)
    if vc_id:
        statement_ids.append(vc_id)

    return statement_ids
