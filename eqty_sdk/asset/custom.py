from os import PathLike
from typing import Any, Optional, Union

from eqty_sdk._rust import Graph as Context
from eqty_sdk.types import Cid

from .asset import Asset, AssetType


class Custom(Asset):
    """Represents a Custom asset with user-specified asset type."""

    @staticmethod
    def from_path(
        path: PathLike[str],
        asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM,
        store: Optional[bool] = None,
        **kwargs,
    ) -> "Asset":
        custom_type = _resolve_type(asset_type)
        return Asset._from_path(path, custom_type, store=store, **kwargs)

    @staticmethod
    def from_cid(
        cid: Union[Cid, str],
        asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM,
        **kwargs,
    ) -> "Asset":
        custom_type = _resolve_type(asset_type)
        cid_str = cid.cid if isinstance(cid, Cid) else cid
        return Asset._from_cid(cid_str, custom_type, ctx=None, **kwargs)

    @staticmethod
    def from_object(
        obj: Any,
        asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM,
        store: Optional[bool] = None,
        **kwargs,
    ) -> "Asset":
        custom_type = _resolve_type(asset_type)
        return Asset._from_object(obj, custom_type, store=store, **kwargs)

    @staticmethod
    def with_context(ctx: Context, asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM):
        custom_type = _resolve_type(asset_type)
        return Asset._factory_with_context(ctx, custom_type)


def _resolve_type(asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM) -> str:
    if isinstance(asset_type, AssetType):
        return asset_type.value
    elif isinstance(asset_type, str):
        return asset_type
    else:
        return AssetType.CUSTOM.value
