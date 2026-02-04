import logging
import os
from typing import List, Optional

from eqty_sdk._rust import (
    Graph as Context,
    statements as eqty_core_statements,
)

from .common import add_vc_statement

logger = logging.getLogger("eqty.sdk.statements")


def add_computation_statement(
    inputs: List[str],
    outputs: List[str],
    computation: Optional[str],
    skip_proof: Optional[bool],
    ctx: Optional[Context] = None,
) -> None:
    """Add a computation registration statement to the integrity graph.

    Args:
        inputs (list[str]): List of input CIDs.
        outputs (list[str]): List of output CIDs.
        computation (str): Computation CID.
        skip_proof (bool): Whether skip the proof.

    """
    timestamp = os.getenv("EQTY_TIMESTAMP", None)
    logger.info(f"creating computation statement. inputs: '{inputs}', outputs: '{outputs}'")
    statement_id = eqty_core_statements.create_computation_statement(
        inputs=inputs,
        outputs=outputs,
        computation=computation,
        timestamp=timestamp,
        graph_id=ctx.id if ctx else None,
    )
    logger.info(f"computation statement '{statement_id}' created")

    add_vc_statement(statement_id, timestamp, skip_proof)
