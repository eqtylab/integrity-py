from .asset import AssetType, TypedAsset


class Model(TypedAsset):
    """Represents a System Prompt asset."""

    _asset_type = AssetType.SYSTEM_PROMPT
