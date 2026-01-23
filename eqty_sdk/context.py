"""Context module with feature flag-based class selection."""

import logging
from typing import Dict, Optional
from uuid import uuid4

from pydantic.types import UUID4

from eqty_sdk._rust import (
    context as eqty_core_context,
)

logger = logging.getLogger("eqty.sdk.context")


class Context:
    """New context implementation used when GRAPH_IDS feature is enabled."""

    def __init__(self, *args, **kwargs):
        raise TypeError(
            "Use Context.new() or Context.from_parent() to create instances of this class."
        )

    def __init_internal__(
        self, name: Optional[str] = None, id: Optional[UUID4] = None
    ) -> "Context":
        uuid = id if id else uuid4()
        logger.info(f"Creating new root context {str(uuid)[-12:]}")
        self.name: str = name if name else str(uuid)
        self.uuid: UUID4 = uuid
        self.parent_ctx: Optional[UUID4] = None
        return self

    @classmethod
    def new(cls, name: Optional[str] = None, id: Optional[UUID4] = None) -> "Context":
        """Creates a new Object with the provided UUID and Name.

        Args:
            name: The name to apply to the context
            id: The UUID to use for the context

        """
        obj = object.__new__(cls)
        ctx = obj.__init_internal__(name, id)
        try:
            eqty_core_context.create_graph_from_context(str(ctx.uuid), ctx.name, None)
        except RuntimeError as e:
            if "UNIQUE constraint failed: graphs.graph_id" in str(e):
                logger.warning(
                    f"Graph {ctx.uuid} already exists in database. The existing name will be used."
                )
            else:
                raise
        return ctx

    @classmethod
    def from_parent(
        cls, ctx: UUID4, name: Optional[str] = None, id: Optional[UUID4] = None
    ) -> "Context":
        """Creates a new context as a child of ctx.

        Args:
            ctx: The UUID of the parent context
            name: Optional name to give to the new context
            id: Optional: The ID to apply to the child context

        """
        logger.info(f"Creating new context from parent {str(ctx)[-12:]}")
        try:
            cls.new(id=ctx)
        except Exception as e:
            logger.info(f"Error creating parent context {e}")

        obj = object.__new__(cls)
        item = obj.__init_internal__(name, id)
        item.parent_ctx = ctx
        try:
            eqty_core_context.create_graph_from_context(str(item.uuid), item.name, str(ctx))
        except RuntimeError as e:
            if "UNIQUE constraint failed: graphs.graph_id" in str(e):
                logger.warning(
                    f"Graph {item.uuid} already exists in database. The parent context was not applied."
                )
            else:
                raise
        return item

    def __repr__(self) -> str:
        return f"Context(name={self.name!r}, uuid={str(self.uuid)[-12:]}, parent_ctx={str(self.parent_ctx)[-12:]})"

    def to_dict(self) -> Dict:
        return {
            "uuid": str(self.uuid)[-12:],
            "name": self.name,
            "parent": str(self.parent_ctx)[-12:],
        }


# Export the classes and factory for external use
__all__ = ["Context"]
