import logging
from functools import wraps
from typing import Any, Callable, Dict, Optional

from eqty_sdk.context import ContextType
from eqty_sdk.feature_flags import FEATURE_FLAGS, FeatureFlags

from . import Compute

logger = logging.getLogger("eqty.sdk.decorator")


def compute(
    metadata: Optional[Dict[str, Any]] = None,
    attributes: Optional[Callable] = None,
    ctx: Optional[ContextType] = None,
    **compute_kwargs,
):
    def decorator(func: Callable):
        @wraps(func)
        def wrapper(*args, **kwargs):
            store = None
            compute_asset = Compute(func, metadata, store, ctx, **compute_kwargs)
            result = compute_asset.__call__(*args, **kwargs)

            if attributes and FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS):
                logger.error(
                    f"Attempted to set attributes with feature flag {FEATURE_FLAGS.GRAPH_IDS} enabled"
                )
            elif attributes:
                attrs: Dict = attributes()
                compute_asset.__add_attribute__(**attrs)

            return result

        return wrapper

    return decorator
