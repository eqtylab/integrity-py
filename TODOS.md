# TODOS

## Docs

### Archive a 2.0 to 2.4 backport in CI

**What:** A workflow on main, started by hand with a version, that does what `just archive-backport` does on a Mac: makes the Linux reports on Linux runners and the macOS reports on macOS runners, runs `scripts/archive_release.sh`, and opens the archive PR through `scripts/open_archive_pr.sh`.

**Why:** A backport cut from 2.0 to 2.4 runs the `release.yml` stored in its tag, which has no archive job, so its docs are archived by hand with `just archive-backport X.Y.Z`. That needs a Mac with Docker, and main's docs build fails until someone runs it.

**Pros:** One click, on any machine, and nobody has to notice the failing build first.

**Cons:** More CI to review and keep working for an event that has not happened yet.

**Context:** `scripts/archive_backport.sh` holds the steps; the workflow would split its report steps across two runners. Run on 2.4.2's published wheels, it rebuilt `archive/v2.4/` with only the reports' whitespace and temporary paths changed.

**Effort:** S
**Priority:** P3
**Depends on:** nothing. Build it when the first backport to 2.0 to 2.4 ships, or before, if one is planned.
