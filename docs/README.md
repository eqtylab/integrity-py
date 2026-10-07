# docs/

The documentation site lives in [`docs-site/`](../docs-site/). This folder holds only
`generated/`:

- `auditwheel-show-linux-*.txt` and `otool-show-macos-*.txt`: the wheel reports the release
  workflow saves for each release, which the Min Version page shows;
- `auditwheel-show-musllinux-*.txt`: saved by the same workflow, shown on no page;
- `built-in-asset-types.md`: written by `just generate-stubs`, shown on the Assets page.

`scripts/render_api_docs.py` copies the shown files into the pages. The archive scripts read
the reports from this path in release tags, so the folder stays here.
