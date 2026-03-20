from os import PathLike
from typing import Optional, Union

from .asset import AssetType, TypedAsset


class Model(TypedAsset):
    """Represents a model asset."""

    _asset_type = AssetType.MODEL

    @classmethod
    def from_path(
        cls,
        path: Union[str, PathLike[str]],
        _store: Optional[bool] = None,
        *,
        _enable_model_signing_signature: bool = False,
        **kwargs,
    ) -> "Model":
        return cls._from_path(
            path,
            cls._asset_type,
            _store=_store,
            _enable_model_signing_signature=_enable_model_signing_signature,
            **kwargs,
        )
