from .asset import AssetType, TypedAsset


class Config(TypedAsset):
    """Represents a config asset."""

    _asset_type = AssetType.CONFIG
