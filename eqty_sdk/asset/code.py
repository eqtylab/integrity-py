from . import AssetType, TypedAsset


class Code(TypedAsset):
    """Represents a code asset."""

    _asset_type = AssetType.CODE
