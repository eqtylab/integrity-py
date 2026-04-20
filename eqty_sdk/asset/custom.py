from os import PathLike
from typing import Any, Optional, Union

from eqty_sdk._rust import (
    CID,
    Context,
)

from .asset import Asset, AssetType


class Custom(Asset):
    """Represents a Custom asset with user-specified asset type."""

    @staticmethod
    def from_path(
        path: Union[str, PathLike[str]],
        asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM,
        _store: Optional[bool] = None,
        **kwargs,
    ) -> "Custom":
        custom_type = _resolve_type(asset_type)
        return Custom._from_path(path, custom_type, _store=_store, **kwargs)

    @staticmethod
    def from_cid(
        cid: CID,
        asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM,
        **kwargs,
    ) -> "Custom":
        custom_type = _resolve_type(asset_type)
        return Custom._from_cid(cid, custom_type, ctx=None, **kwargs)

    @staticmethod
    def from_object(
        obj: Any,
        asset_type: Optional[Union[AssetType, str]] = AssetType.CUSTOM,
        _store: Optional[bool] = None,
        **kwargs,
    ) -> "Custom":
        custom_type = _resolve_type(asset_type)
        return Custom._from_object(obj, custom_type, _store=_store, **kwargs)

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
