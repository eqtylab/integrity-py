# Service Registering

This example creates a graph in a specific Governance Studio project and registers the graph, statements, and blobs with a service endpoint.

Source: `examples/service-registering.py`

```python
--8<-- "examples/service-registering.py"
```

Important details:

- `Context.from_uuid(...)` takes the UUID for the Governance Studio project where you want this graph tree to be registered.
- `Context.with_parent(gov_ctx).new("My Agent v1")` creates a child graph under that Governance Studio project, and the name passed to `.new(...)` is the label that Governance Studio shows for the child graph.
- `Service.new(...)` uses the `EQTY_API_KEY` environment variable when an API key is not passed directly. Set `EQTY_API_KEY` to an API key created in Governance Studio before running this example.
- If you expect to register blobs to a remote service often, pair this pattern with `cfg.set_store_all_blobs(True)` as shown in `Remote Service Workflow`.
