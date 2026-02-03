import logging
from functools import wraps
from typing import Any, Callable, Dict, Optional

from eqty_sdk._rust import Graph as Context

from . import Compute

logger = logging.getLogger("eqty.sdk.decorator")


def compute(
    metadata: Optional[Dict[str, Any]] = None,
    ctx: Optional[Context] = None,
    **compute_kwargs,
):
    def decorator(func: Callable):
        @wraps(func)
        def wrapper(*args, **kwargs):
            store = None
            compute_asset = Compute(func, metadata, store, ctx, **compute_kwargs)
            result = compute_asset.__call__(*args, **kwargs)

            return result

        return wrapper

    return decorator
