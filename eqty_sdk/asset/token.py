from .asset import AssetType, TypedAsset


class Token(TypedAsset):
    """Represents a token asset."""

    _asset_type = AssetType.TOKEN
