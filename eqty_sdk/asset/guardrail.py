from .asset import AssetType, TypedAsset


class Guardrail(TypedAsset):
    """Represents a guardrail asset."""

    _asset_type = AssetType.GUARDRAIL
