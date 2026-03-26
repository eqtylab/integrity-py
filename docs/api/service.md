# Service

A service is the SDK's connection to a remote Integrity Service or Governance Studio-backed API.

You should use `Service` when you want to move beyond local graph generation and start registering contexts, statements, and blobs with a remote system. That is the path for sharing graphs, registering them to Governance Studio projects, or making locally generated provenance available to other systems.

Typical usage is:

- create a `Service` with the remote base URL
- provide an API key directly or through `EQTY_API_KEY` when required by your workflow
- call `Context.register(service)` or related APIs to push the graph and referenced blobs

If your workflow is only generating local manifests, you may not need `Service` at all.

::: eqty_sdk._rust.Service
    options:
      members_order: source
