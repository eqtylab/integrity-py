from pathlib import Path
from typing import Any, Optional, Union, cast

from eqty_sdk._rust import Graph as Context
from eqty_sdk.config.config import Config
from eqty_sdk.types.cid import Cid

from .asset import Asset, AssetType


class Database(Asset):
    """Represents a database asset."""

    @staticmethod
    def from_path(path: Union[str, Path], store: Optional[bool] = None, **kwargs) -> "Database":
        return cast(
            "Database",
            Asset._from_path(Config().root_context, path, AssetType.DATABASE, store, **kwargs),
        )

    @staticmethod
    def from_cid(cid: Union[Cid, str], **kwargs) -> "Database":
        if isinstance(cid, Cid):
            return cast(
                "Database",
                Asset._from_cid(Config().root_context, cid.cid, AssetType.DATABASE, **kwargs),
            )
        else:
            return cast(
                "Database",
                Asset._from_cid(Config().root_context, cid, AssetType.DATABASE, **kwargs),
            )

    @staticmethod
    def from_object(obj: Any, store: Optional[bool] = None, **kwargs) -> "Database":
        return cast(
            "Database",
            Asset._from_object(Config().root_context, obj, AssetType.DATABASE, store, **kwargs),
        )

    @staticmethod
    def with_context(ctx: Context):
        return Asset._factory_with_context(ctx, AssetType.DATABASE)
