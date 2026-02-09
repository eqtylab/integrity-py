import logging
import os
from typing import List, Optional

from eqty_sdk._rust import statements as eqty_core_statements

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_computation_statement(
    inputs: List[str],
    outputs: List[str],
    computation: Optional[str],
    skip_proof: Optional[bool],
) -> List[str]:
    """Add a computation registration statement to the integrity graph.

    Args:
        inputs (list[str]): List of input CIDs.
        outputs (list[str]): List of output CIDs.
        computation (str): Computation CID.
        skip_proof (bool): Whether skip the proof.

    Returns:
        List[str]: The list of IDs of the created statements.

    """
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    logger.info(f"creating computation statement. inputs: '{inputs}', outputs: '{outputs}'")
    statement_id = eqty_core_statements.create_computation_statement(
        inputs=inputs, outputs=outputs, computation=computation, timestamp=timestamp
    )
    logger.info(f"computation statement '{statement_id}' created")
    statement_ids = [statement_id]

    vc_id = add_vc_statement(statement_id, timestamp, skip_proof)
    if vc_id:
        statement_ids.append(vc_id)

    return statement_ids
