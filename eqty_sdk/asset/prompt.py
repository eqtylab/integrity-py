from .asset import AssetType, TypedAsset


class Prompt(TypedAsset):
    """Represents a Prompt asset."""

    _asset_type = AssetType.PROMPT
