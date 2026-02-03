from .asset import AssetType, TypedAsset


class Database(TypedAsset):
    """Represents a database asset."""

    _asset_type = AssetType.DATABASE
