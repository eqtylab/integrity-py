from pathlib import Path
from typing import Any, Optional, Union, cast

from eqty_sdk.config.config import Config
from eqty_sdk.context import Context, ContextType
from eqty_sdk.feature_flags import FEATURE_FLAGS, FeatureFlags, feature_gate_when_disabled
from eqty_sdk.types.cid import Cid

from .asset import Asset, AssetType


class Token(Asset):
    """Represents a token asset."""

    @staticmethod
    def from_path(path: Union[str, Path], store: Optional[bool] = None, **kwargs) -> "Token":
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )
        return cast("Token", Asset._from_path(ctx, path, AssetType.TOKEN, store, **kwargs))

    @staticmethod
    def from_cid(cid: Union[Cid, str], **kwargs) -> "Token":
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )
        if isinstance(cid, Cid):
            return cast("Token", Asset._from_cid(ctx, cid.cid, AssetType.TOKEN, **kwargs))
        else:
            return cast("Token", Asset._from_cid(ctx, cid, AssetType.TOKEN, **kwargs))

    @staticmethod
    def from_object(obj: Any, store: Optional[bool] = None, **kwargs) -> "Token":
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )
        return cast("Token", Asset._from_object(ctx, obj, AssetType.TOKEN, store, **kwargs))

    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def add_attribute(self, **kwargs) -> "Token":
        self.add_attribute(**kwargs)
        return self

    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def remove_attribute(self, **kwargs) -> "Token":
        self.remove_attribute(**kwargs)
        return self

    @staticmethod
    def with_context(ctx: ContextType):
        return Asset._factory_with_context(ctx, AssetType.TOKEN)
