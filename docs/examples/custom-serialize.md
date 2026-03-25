# Spark Parquet Input

For Spark customers, the recommended pattern is to track the parquet source as the input asset rather than hashing the in-memory Spark DataFrame object.

Source: `examples/custom-serialize.py`

```python
--8<-- "examples/custom-serialize.py"
```

This works because compute inputs that implement `to_eqty_asset()` are converted into SDK assets before the computation is registered.
