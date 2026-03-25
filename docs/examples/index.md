# Examples

These examples show how to use the SDK end to end instead of one API surface at a time.

- `Basic Workflow`: initialize the SDK, create input assets, define a `@compute` function, execute it, and export a manifest.
- `Async Compute`: use `@compute` with async functions and async generators.
- `Service Registering`: create a graph for a Governance Studio project and register it with a service endpoint.
- `Spark Parquet Input`: wrap a Spark DataFrame so provenance is attached to the parquet source on disk.
- `Model Signing`: opt in to model-signing statements for model directories.

The source files live under `examples/`.
