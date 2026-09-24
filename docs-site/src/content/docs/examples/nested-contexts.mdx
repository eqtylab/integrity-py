# Nested Contexts

Use this example when a Governance Studio project should be the root of the graph tree, but you still want local subcontexts below it for project-level state and individual runs.

Context graphs are built hierarchically. If data is registered in a parent graph and then referenced by CID in a child graph, the child graph can automatically pull that parent data into its own lineage. This is useful for keeping higher-level shared assets, such as project specs, prompts, or baseline models, in a parent context while allowing multiple child contexts to reuse them without re-registering the same logical object in each graph.

## Create The Shared Model

Source: `examples/creating-the-model.py`

```python
--8<-- "examples/creating-the-model.py"
```

## Use The Shared Model In A Child Context

Source: `examples/using-the-model.py`

```python
--8<-- "examples/using-the-model.py"
```

Notes:

- `Context.from_uuid(...)` takes the Governance Studio project UUID that should be the root of the graph tree.
- `Context.with_parent(gov_ctx).new("...")` creates a local child context under the Governance Studio project. That child becomes the default context for the process in this example.
- `Context.with_parent(cfg.get_default_context()).new("...")` creates a run-specific child context, which is therefore a grandchild of the Governance Studio project.
- The first script registers the default subcontext and its shared model to `http://localhost:3050`.
- The second script creates a run context under that default subcontext, references the shared model by CID, and registers the run context to the same service.
