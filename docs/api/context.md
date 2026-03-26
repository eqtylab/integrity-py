# Context

A context is a logical graph container. It groups related statements, assets, and computations so they can be exported, viewed, or registered together.

Contexts matter because they define the boundary of a manifest and the unit of organization for a workflow. If you want to keep different projects, runs, customers, or Governance Studio graphs separate, use different contexts. Parent-child contexts are useful when you want multiple runs or subgraphs to roll up under a larger project-level graph.

Most users should create a context early, pass it to `init(default_context=...)`, and then let assets and computations attach to that context by default. Use `Context.from_uuid(...)` when you want the graph to correspond to an existing Governance Studio project, and use `export(...)`, `import_manifest(...)`, or `register(...)` when moving the graph between local state and external systems.

::: eqty_sdk._rust.Context
    options:
      members_order: source
