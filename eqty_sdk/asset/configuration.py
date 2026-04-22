from .asset import AssetType, TypedAsset


class Configuration(TypedAsset):
    """Represents a configuration asset."""

    _asset_type = AssetType.CONFIGURATION
