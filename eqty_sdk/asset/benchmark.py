from pathlib import Path
from typing import Any, Optional, Union, cast

from eqty_sdk.config.config import Config
from eqty_sdk.context import Context, ContextType
from eqty_sdk.feature_flags import FEATURE_FLAGS, FeatureFlags, feature_gate_when_disabled
from eqty_sdk.types.cid import Cid

from .asset import Asset, AssetType


class Benchmark(Asset):
    """Represents a benchmark asset."""

    @staticmethod
    def from_path(path: Union[str, Path], store: Optional[bool] = None, **kwargs) -> "Benchmark":
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )
        return cast("Benchmark", Asset._from_path(ctx, path, AssetType.BENCHMARK, store, **kwargs))

    @staticmethod
    def from_cid(cid: Union[Cid, str], **kwargs) -> "Benchmark":
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )
        if isinstance(cid, Cid):
            return cast("Benchmark", Asset._from_cid(ctx, cid.cid, AssetType.BENCHMARK, **kwargs))
        else:
            return cast("Benchmark", Asset._from_cid(ctx, cid, AssetType.BENCHMARK, **kwargs))

    @staticmethod
    def from_object(obj: Any, store: Optional[bool] = None, **kwargs) -> "Benchmark":
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )
        return cast("Benchmark", Asset._from_object(ctx, obj, AssetType.BENCHMARK, store, **kwargs))

    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def add_attribute(self, **kwargs) -> "Benchmark":
        self.add_attribute(**kwargs)
        return self

    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def remove_attribute(self, **kwargs) -> "Benchmark":
        self.remove_attribute(**kwargs)
        return self

    @staticmethod
    def with_context(ctx: ContextType):
        return Asset._factory_with_context(ctx, AssetType.BENCHMARK)


class BenchmarkResult(Asset):
    """Represents a benchmark result asset."""

    @staticmethod
    def from_path(
        path: Union[str, Path], store: Optional[bool] = None, **kwargs
    ) -> "BenchmarkResult":
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )
        return cast(
            "BenchmarkResult",
            Asset._from_path(ctx, path, AssetType.BENCHMARK_RESULT, store, **kwargs),
        )

    @staticmethod
    def from_cid(cid: Union[Cid, str], **kwargs) -> "BenchmarkResult":
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )
        if isinstance(cid, Cid):
            return cast(
                "BenchmarkResult",
                Asset._from_cid(ctx, cid.cid, AssetType.BENCHMARK_RESULT, **kwargs),
            )
        else:
            return cast(
                "BenchmarkResult",
                Asset._from_cid(ctx, cid, AssetType.BENCHMARK_RESULT, **kwargs),
            )

    @staticmethod
    def from_object(obj: Any, store: Optional[bool] = None, **kwargs) -> "BenchmarkResult":
        ctx = (
            Config().root_context if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS) else Context()
        )
        return cast(
            "BenchmarkResult",
            Asset._from_object(ctx, obj, AssetType.BENCHMARK_RESULT, store, **kwargs),
        )

    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def add_attribute(self, **kwargs) -> "BenchmarkResult":
        self.add_attribute(**kwargs)
        return self

    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def remove_attribute(self, **kwargs) -> "BenchmarkResult":
        self.remove_attribute(**kwargs)
        return self

    @staticmethod
    def with_context(ctx: ContextType):
        return Asset._factory_with_context(ctx, AssetType.BENCHMARK_RESULT)
