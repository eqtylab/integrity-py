# Spark Parquet Input

For Spark customers, the recommended pattern is to track the parquet source as the input asset rather than hashing the in-memory Spark DataFrame object.

Source: `examples/custom-serialize.py`

```python
from dataclasses import dataclass
from pathlib import Path

from eqty_sdk import Dataset, Signer, compute, init, set_active_signer
from pyspark.sql import DataFrame, SparkSession


@dataclass
class SparkParquetFrame:
    dataframe: DataFrame
    parquet_path: Path
    name: str

    def to_eqty_asset(self) -> Dataset:
        return Dataset.from_path(
            self.parquet_path,
            name=self.name,
            description="Spark DataFrame backed by a parquet dataset",
        )


@compute(
    metadata={
        "description": "Pass a Spark DataFrame while tracking the parquet source asset.",
    }
)
def count_rows(frame: SparkParquetFrame) -> int:
    return frame.dataframe.count()
```

This works because compute inputs that implement `to_eqty_asset()` are converted into SDK assets before the computation is registered.
