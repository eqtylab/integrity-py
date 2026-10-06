# TODOS

## Docs

### Retire the MkDocs site

**What:** Delete the MkDocs setup the docs no longer use:

- `mkdocs.yml`
- the MkDocs sources in `docs/`: `index.md`, `quick-start.md`, `api/`, `examples/`, `install/`, `stylesheets/` and `javascripts/`. Keep `docs/generated/`, which the site's reference pages and the release job's archive step still read.
- the `docs` group in `pyproject.toml` (`mkdocs`, `mkdocstrings`, `pymdown-extensions`) and its entries in `poetry.lock`
- `--with docs` in the Justfile's `install` recipe
- the Justfile's `build-docs` and `serve-docs` recipes, and their lines in README's command list and project structure

Keep the `gh-pages` branch, though Pages no longer publishes it: the docs build check's `scripts/check_old_links.py` and `scripts/archive_version.py` read the old site from its commit `cb7be3c`.

**Why:** Nothing publishes MkDocs any more. The site is `docs-site/` on Vercel, and GitHub Pages serves only the forwarder, so `docs/` is a second copy of the prose that can drift from `docs-site/src/content/docs/`.

**Pros:** One source for the docs. A smaller `just install`, without three packages nobody uses.

**Cons:** It changes the SDK team's local setup. Anyone used to `just serve-docs` moves to `pnpm dev` in `docs-site/`.

**Context:** It was kept out of the GitHub Pages forwarder PR, which only stops the old site being published. Nobody has edited `docs/` outside `generated/` since 2026-09-24, when `docs-site/` copied it. `scripts/archive_version.py` converts old releases from their tags, not from this checkout, so it does not need these files.

**Effort:** S
**Priority:** P3
**Depends on:** the GitHub Pages forwarder PR merged.

### Archive a 2.0 to 2.4 backport in CI

**What:** A workflow on main, started by hand with a version, that does what `just archive-backport` does on a Mac: makes the Linux reports on Linux runners and the macOS reports on macOS runners, runs `scripts/archive_release.sh`, and opens the archive PR through `scripts/open_archive_pr.sh`.

**Why:** A backport cut from 2.0 to 2.4 runs the `release.yml` stored in its tag, which has no archive job, so its docs are archived by hand with `just archive-backport X.Y.Z`. That needs a Mac with Docker, and main's docs build fails until someone runs it.

**Pros:** One click, on any machine, and nobody has to notice the failing build first.

**Cons:** More CI to review and keep working for an event that has not happened yet.

**Context:** `scripts/archive_backport.sh` holds the steps; the workflow would split its report steps across two runners. Run on 2.4.2's published wheels, it rebuilt `archive/v2.4/` with only the reports' whitespace and temporary paths changed.

**Effort:** S
**Priority:** P3
**Depends on:** nothing. Build it when the first backport to 2.0 to 2.4 ships, or before, if one is planned.
