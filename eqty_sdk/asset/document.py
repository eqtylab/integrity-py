from .asset import AssetType, TypedAsset


class Document(TypedAsset):
    """Represents a document asset."""

    _asset_type = AssetType.DOCUMENT
