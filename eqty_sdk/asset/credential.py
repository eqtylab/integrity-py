from .asset import AssetType, TypedAsset


class Credential(TypedAsset):
    """Represents a credential asset."""

    _asset_type = AssetType.CREDENTIAL
