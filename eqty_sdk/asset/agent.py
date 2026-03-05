from .asset import AssetType, TypedAsset


class Agent(TypedAsset):
    """Represents an AI Agent."""

    _asset_type = AssetType.AGENT
