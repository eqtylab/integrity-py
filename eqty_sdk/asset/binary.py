from .asset import AssetType, TypedAsset


class Binary(TypedAsset):
    """Represents a compiled binary program asset."""

    _asset_type = AssetType.BINARY
