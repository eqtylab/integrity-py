from contextlib import contextmanager
from contextvars import ContextVar
from typing import Iterator, Optional

from eqty_sdk._rust import Context

_active_ctx: ContextVar[Optional[Context]] = ContextVar("eqty_active_ctx", default=None)


def get_active_context() -> Optional[Context]:
    return _active_ctx.get()


@contextmanager
def graph_context(ctx: Context) -> Iterator[None]:
    """Temporarily set the active graph context for SDK calls in this block.

    Example:
        from eqty_sdk import Context, Dataset
        from eqty_sdk.context import graph_context

        ctx = Context.new("my-graph")
        with graph_context(ctx):
            Dataset.from_object("hello", name="greeting")

    """
    token = _active_ctx.set(ctx)
    try:
        yield
    finally:
        _active_ctx.reset(token)
