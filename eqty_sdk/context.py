from contextlib import contextmanager
from contextvars import ContextVar
from typing import Iterator, Optional

from eqty_sdk._rust import Context

_active_ctx: ContextVar[Optional[Context]] = ContextVar("eqty_active_ctx", default=None)


def get_active_context() -> Optional[Context]:
    return _active_ctx.get()


@contextmanager
def graph_context(ctx: Context) -> Iterator[None]:
    token = _active_ctx.set(ctx)
    try:
        yield
    finally:
        _active_ctx.reset(token)
