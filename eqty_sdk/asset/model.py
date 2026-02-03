from .asset import AssetType, TypedAsset


class Model(TypedAsset):
    """Represents a model asset."""

    _asset_type = AssetType.MODEL
