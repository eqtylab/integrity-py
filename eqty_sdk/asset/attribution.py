from .asset import AssetType, TypedAsset


class Attribution(TypedAsset):
    """Represents an attribution asset."""

    _asset_type = AssetType.ATTRIBUTION
