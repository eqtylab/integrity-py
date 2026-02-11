import json
import logging
from enum import Enum
from os import PathLike
from typing import Any, Optional, Union, cast

import dill as pickle

from eqty_sdk._rust import (
    Asset as Asset,
    Graph as Context,
)
from eqty_sdk.types import Cid

logger = logging.getLogger("eqty.sdk.Asset")


class AssetType(Enum):
    ATTRIBUTION = "Attribution"
    BENCHMARK = "Benchmark"
    BENCHMARK_RESULT = "Benchmark_Result"
    CERTIFICATE = "Certificate"
    CODE = "Code"
    CUSTOM = "Custom"
    DATABASE = "Database"
    DATASET = "Dataset"
    DOCUMENT = "Document"
    MEDIA = "Media"
    MODEL = "Model"
    TOKEN = "Token"


def serialize_for_hashing(obj: Any) -> bytes:
    if isinstance(obj, str):
        return obj.encode("utf-8")
    if isinstance(obj, (int, float)):
        return str(obj).encode("utf-8")
    if isinstance(obj, list) or isinstance(obj, dict):
        return json.dumps(obj).encode("utf-8")
    if hasattr(obj, "serialize_for_hashing"):
        return cast(bytes, obj.serialize_for_hashing())
    try:
        if hasattr(obj, "model"):
            state_dict = getattr(obj, "model").state_dict()
            return cast(bytes, pickle.dumps(state_dict))
        return cast(bytes, pickle.dumps(obj))
    except (pickle.PickleError, TypeError) as e:
        raise TypeError(f"Unsupported data type for hashing: {type(obj)} - {e}")


def get_asset_name(asset_type: Union[AssetType, str], cid: str) -> str:
    if isinstance(asset_type, AssetType):
        return f"{asset_type.value}-{cid[-4:]}"
    return f"{asset_type}-{cid[-4:]}"


class TypedAsset:
    _asset_type: AssetType

    @classmethod
    def from_path(cls, path: PathLike[str], store: Optional[bool] = None, **kwargs) -> Asset:
        return Asset._from_path(path, cls._asset_type, store=store, **kwargs)

    @classmethod
    def from_cid(cls, cid: Union[Cid, str], **kwargs) -> Asset:
        cid_str = cid.cid if isinstance(cid, Cid) else cid
        return Asset._from_cid(cid_str, cls._asset_type, **kwargs)

    @classmethod
    def from_object(cls, obj: Any, store: Optional[bool] = None, **kwargs) -> Asset:
        return Asset._from_object(obj, cls._asset_type, store=store, **kwargs)

    @classmethod
    def with_context(cls, ctx: Context) -> Any:
        return Asset._factory_with_context(ctx, cls._asset_type)


class _Factory:
    def __init__(self, ctx: Context, asset_type: Union[AssetType, str]):
        self._ctx = ctx
        self._asset_type = asset_type

    def from_path(self, path: PathLike[str], store: Optional[bool] = None, **kwargs) -> Asset:
        return Asset._from_path(path, self._asset_type, self._ctx, store, **kwargs)

    def from_cid(self, cid: Union[Cid, str], **kwargs) -> Asset:
        cid_str = cid.cid if isinstance(cid, Cid) else cid
        return Asset._from_cid(cid_str, self._asset_type, self._ctx, **kwargs)

    def from_object(self, obj: Any, store: Optional[bool] = None, **kwargs) -> Asset:
        return Asset._from_object(obj, self._asset_type, self._ctx, store, **kwargs)


from .attribution import Attribution  # noqa: E402
from .benchmark import Benchmark, BenchmarkResult  # noqa: E402
from .certificate import Certificate  # noqa: E402
from .code import Code  # noqa: E402
from .custom import Custom  # noqa: E402
from .database import Database  # noqa: E402
from .dataset import Dataset  # noqa: E402
from .document import Document  # noqa: E402
from .media import Media  # noqa: E402
from .model import Model  # noqa: E402
from .token import Token  # noqa: E402

__all__ = [
    "Asset",
    "AssetType",
    "Attribution",
    "Benchmark",
    "BenchmarkResult",
    "Certificate",
    "Code",
    "Custom",
    "Database",
    "Dataset",
    "Document",
    "Media",
    "Model",
    "Token",
    "serialize_for_hashing",
]
