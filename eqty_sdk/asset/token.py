from pathlib import Path
from typing import Any, Optional, Union, cast

from eqty_sdk.config.config import Config
from eqty_sdk.context import Context
from eqty_sdk.types.cid import Cid

from .asset import Asset, AssetType


class Token(Asset):
    """Represents a token asset."""

    @staticmethod
    def from_path(path: Union[str, Path], store: Optional[bool] = None, **kwargs) -> "Token":
        return cast(
            "Token", Asset._from_path(Config().root_context, path, AssetType.TOKEN, store, **kwargs)
        )

    @staticmethod
    def from_cid(cid: Union[Cid, str], **kwargs) -> "Token":
        if isinstance(cid, Cid):
            return cast(
                "Token", Asset._from_cid(Config().root_context, cid.cid, AssetType.TOKEN, **kwargs)
            )
        else:
            return cast(
                "Token", Asset._from_cid(Config().root_context, cid, AssetType.TOKEN, **kwargs)
            )

    @staticmethod
    def from_object(obj: Any, store: Optional[bool] = None, **kwargs) -> "Token":
        return cast(
            "Token",
            Asset._from_object(Config().root_context, obj, AssetType.TOKEN, store, **kwargs),
        )

    @staticmethod
    def with_context(ctx: Context):
        return Asset._factory_with_context(ctx, AssetType.TOKEN)
