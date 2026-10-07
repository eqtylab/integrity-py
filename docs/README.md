# docs/

The documentation site lives in [`docs-site/`](../docs-site/). This folder holds only
`generated/`: the wheel reports the release workflow saves for each release, and
`built-in-asset-types.md`, which `just generate-stubs` writes. `scripts/render_api_docs.py`
copies them into the pages. The archive scripts read this path from the 2.0 to 2.4 release
tags, so the folder stays here.
