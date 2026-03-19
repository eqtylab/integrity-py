from .asset import AssetType, TypedAsset


class Skill(TypedAsset):
    """Represents a skill asset."""

    _asset_type = AssetType.SKILL
