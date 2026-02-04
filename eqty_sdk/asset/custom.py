from pathlib import Path
from typing import Any, Optional, Union, cast

from eqty_sdk import config
from eqty_sdk._rust import Graph as Context
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

        return cast(
            "Custom", Asset._from_path(config.root_context(), path, custom_type, store, **kwargs)
        )

    @staticmethod
    def from_cid(
        cid: Union[Cid, str],
        asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM,
        **kwargs,
    ) -> "Custom":
        custom_type = _resolve_type(asset_type)

        if isinstance(cid, Cid):
            return cast(
                "Custom", Asset._from_cid(config.root_context(), cid.cid, custom_type, **kwargs)
            )
        else:
            return cast(
                "Custom", Asset._from_cid(config.root_context(), cid, custom_type, **kwargs)
            )

    @staticmethod
    def from_object(
        obj: Any,
        asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM,
        store: Optional[bool] = None,
        **kwargs,
    ) -> "Custom":
        custom_type = _resolve_type(asset_type)

        return cast(
            "Custom", Asset._from_object(config.root_context(), obj, custom_type, store, **kwargs)
        )

    @staticmethod
    def with_context(ctx: Context):
        return Asset._factory_with_context(ctx, AssetType.CUSTOM)


def _resolve_type(asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM):
    if isinstance(asset_type, AssetType):
        return asset_type.value
    elif isinstance(asset_type, str):
        return asset_type
    else:
        return AssetType.CUSTOM.value
