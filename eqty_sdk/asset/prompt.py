from .asset import AssetType, TypedAsset


class Model(TypedAsset):
    """Represents a Prompt asset."""

    _asset_type = AssetType.PROMPT
