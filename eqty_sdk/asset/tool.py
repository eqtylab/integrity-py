from .asset import AssetType, TypedAsset


class Tool(TypedAsset):
    """Represents a tool asset."""

    _asset_type = AssetType.TOOL
