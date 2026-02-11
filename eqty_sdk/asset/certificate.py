from . import AssetType, TypedAsset


class Certificate(TypedAsset):
    """Represents a certificate asset."""

    _asset_type = AssetType.CERTIFICATE
