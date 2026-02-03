from pathlib import Path
from typing import Any, Optional, Union, cast

from eqty_sdk._rust import Graph as Context
from eqty_sdk.types.cid import Cid

from .asset import Asset, AssetType


class Benchmark(Asset):
    """Represents a benchmark asset."""

    @staticmethod
    def from_path(path: Union[str, Path], store: Optional[bool] = None, **kwargs) -> "Benchmark":
        return cast(
            "Benchmark",
            Asset._from_path(path, AssetType.BENCHMARK, store=store, **kwargs),
        )

    @staticmethod
    def from_cid(cid: Union[Cid, str], **kwargs) -> "Benchmark":
        if isinstance(cid, Cid):
            return cast(
                "Benchmark",
                Asset._from_cid(cid.cid, AssetType.BENCHMARK, **kwargs),
            )
        else:
            return cast(
                "Benchmark",
                Asset._from_cid(cid, AssetType.BENCHMARK, **kwargs),
            )

    @staticmethod
    def from_object(obj: Any, store: Optional[bool] = None, **kwargs) -> "Benchmark":
        return cast(
            "Benchmark",
            Asset._from_object(obj, AssetType.BENCHMARK, store=store, **kwargs),
        )

    @staticmethod
    def with_context(ctx: Context):
        return Asset._factory_with_context(ctx, AssetType.BENCHMARK)


class BenchmarkResult(Asset):
    """Represents a benchmark result asset."""

    @staticmethod
    def from_path(
        path: Union[str, Path], store: Optional[bool] = None, **kwargs
    ) -> "BenchmarkResult":
        return cast(
            "BenchmarkResult",
            Asset._from_path(path, AssetType.BENCHMARK_RESULT, store, **kwargs),
        )

    @staticmethod
    def from_cid(cid: Union[Cid, str], **kwargs) -> "BenchmarkResult":
        if isinstance(cid, Cid):
            return cast(
                "BenchmarkResult",
                Asset._from_cid(cid.cid, AssetType.BENCHMARK_RESULT, **kwargs),
            )
        else:
            return cast(
                "BenchmarkResult",
                Asset._from_cid(cid, AssetType.BENCHMARK_RESULT, **kwargs),
            )

    @staticmethod
    def from_object(obj: Any, store: Optional[bool] = None, **kwargs) -> "BenchmarkResult":
        return cast(
            "BenchmarkResult",
            Asset._from_object(obj, AssetType.BENCHMARK_RESULT, store, **kwargs),
        )

    @staticmethod
    def with_context(ctx: Context):
        return Asset._factory_with_context(ctx, AssetType.BENCHMARK_RESULT)
