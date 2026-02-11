from . import AssetType, TypedAsset


class Media(TypedAsset):
    """Represents a media asset."""

    _asset_type = AssetType.MEDIA
