from .asset import AssetType, TypedAsset


class Benchmark(TypedAsset):
    """Represents a benchmark asset."""

    _asset_type = AssetType.BENCHMARK


class BenchmarkResult(TypedAsset):
    """Represents a benchmark result asset."""

    _asset_type = AssetType.BENCHMARK_RESULT
