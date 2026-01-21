import logging
import os
from typing import Any, Dict, List, Optional, cast
from uuid import UUID

from pydantic.types import UUID4

from eqty_sdk._rust import statements as eqty_core_statements
from eqty_sdk.config.config import Config
from eqty_sdk.feature_flags import FEATURE_FLAGS, feature_gate, feature_gate_when_disabled

logger = logging.getLogger("eqty.sdk.statements")


def add_vc_statement(
    subject: str,
    timestamp: Optional[str],
    skip_proof: Optional[bool],
) -> Optional[str]:
    """Creates a VC statement for the prvided subject ONLY if skip_proof is false.
    If a VC Statement is created, the statement id is returned.
    """
    if skip_proof or (skip_proof is None and os.getenv("EQTY_SKIP_PROOF", "").lower() == "true"):
        logger.info("Skipping issuing of VC")
        return None

    statement_id = eqty_core_statements.create_vc_statement(subject, timestamp)

    return cast(str, statement_id)


class Statements:
    def __init__(self):
        self.statements: List[Dict[str, Any]] = []
        self.attributes: Dict[str, Any] = {}
        self.graphs: List[Dict[str, Any]] = []

    # Selects the statements with the provided attributes
    @classmethod
    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def select(cls, query_string) -> "Statements":
        instance = cls()
        (statements, attributes) = eqty_core_statements.retrieve_statements(query_string)
        logger.info(f"Selected '{len(statements)}' statements")
        instance.statements = statements
        instance.attributes = attributes
        return instance

    @classmethod
    @feature_gate(FEATURE_FLAGS.GRAPH_IDS)
    def select_graph(cls, graph_id: Optional[list[UUID4] | UUID4] = None) -> "Statements":
        """Returns a Statements object populated with statements for the provided graph_id.

        Args:
            graph_id: Optional[UUID4] - The graph_id to get statements for. If not provided, the root context  graph_id is used.

        """
        instance = cls()
        graph_ids = []
        if not graph_id:
            graph_ids = [str(Config().root_context.uuid)]
        elif isinstance(graph_id, UUID):
            graph_ids = [str(graph_id)]
        elif isinstance(graph_id, list):
            for id in graph_id:
                graph_ids.append(str(id))

        logger.info(f"Getting statements for graph_id {graph_ids}")

        graphs = eqty_core_statements.retrieve_graph(graph_ids)
        instance.graphs = graphs
        return instance

    # Selects ALL the statements
    @classmethod
    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def select_all(cls) -> "Statements":
        instance = cls()
        (statements, attributes) = eqty_core_statements.retrieve_statements()
        logger.info(f"Selected '{len(statements)}' total statements")
        instance.statements = statements
        instance.attributes = attributes
        return instance

    @staticmethod
    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def delete(query_string) -> None:
        logger.info(f"Deleting statements where {query_string}")
        eqty_core_statements.delete_statements(query_string)

    @staticmethod
    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def delete_all() -> None:
        logger.info("Deleting ALL statements")
        eqty_core_statements.delete_statements()
