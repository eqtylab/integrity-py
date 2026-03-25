# Context Linking

Use multiple contexts when you want separate graphs for a root project and a child run, while still keeping the lineage connected through parent-child graph relationships.

Source: `examples/context-linking.py`

```python
--8<-- "examples/context-linking.py"
```

Notes:

- `Context.new(...)` creates a fresh local graph with a generated UUID.
- `Context.with_parent(parent).new(...)` creates a child graph that stays linked to the parent in the graph tree.
- `Dataset.with_context(ctx)` and `Computation.with_context(ctx)` let you attach statements to a specific context without changing the default context for the whole process.
