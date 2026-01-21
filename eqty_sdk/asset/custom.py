from pathlib import Path
from typing import Any, Optional, Union, cast

from eqty_sdk.config.config import Config
from eqty_sdk.context import Context, ContextType
from eqty_sdk.feature_flags import FEATURE_FLAGS, FeatureFlags, feature_gate_when_disabled
from eqty_sdk.types.cid import Cid

from .asset import Asset, AssetType


class Custom(Asset):
    """Represents a Custom asset."""

    @staticmethod
    def from_path(
        path: Union[str, Path],
        asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM,
        store: Optional[bool] = None,
        **kwargs,
    ) -> "Custom":
        custom_type = _resolve_type(asset_type)
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )

        return cast("Custom", Asset._from_path(ctx, path, custom_type, store, **kwargs))

    @staticmethod
    def from_cid(
        cid: Union[Cid, str],
        asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM,
        **kwargs,
    ) -> "Custom":
        custom_type = _resolve_type(asset_type)
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )

        if isinstance(cid, Cid):
            return cast("Custom", Asset._from_cid(ctx, cid.cid, custom_type, **kwargs))
        else:
            return cast("Custom", Asset._from_cid(ctx, cid, custom_type, **kwargs))

    @staticmethod
    def from_object(
        obj: Any,
        asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM,
        store: Optional[bool] = None,
        **kwargs,
    ) -> "Custom":
        custom_type = _resolve_type(asset_type)
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )

        return cast("Custom", Asset._from_object(ctx, obj, custom_type, store, **kwargs))

    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def add_attribute(self, **kwargs) -> "Custom":
        self.add_attribute(**kwargs)
        return self

    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def remove_attribute(self, **kwargs) -> "Custom":
        self.remove_attribute(**kwargs)
        return self

    @staticmethod
    def with_context(ctx: ContextType):
        return Asset._factory_with_context(ctx, AssetType.CUSTOM)


def _resolve_type(asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM):
    if isinstance(asset_type, AssetType):
        return asset_type.value
    elif isinstance(asset_type, str):
        return asset_type
    else:
        return AssetType.CUSTOM.value
