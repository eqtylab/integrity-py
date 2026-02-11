from . import AssetType, TypedAsset


class Dataset(TypedAsset):
    """Represents a dataset asset."""

    _asset_type = AssetType.DATASET
