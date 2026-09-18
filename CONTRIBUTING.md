# Contributing to integrity-py

integrity-py is open source, but we are not accepting outside contributions at this
time. Please do not submit external pull requests.

The guidance below is for the internal maintainers working on changes and releases.

Build prerequisites and development checks are in [README.md](README.md#development).
Maintain [CHANGELOG.md](CHANGELOG.md) as part of PR review and release preparation,
following [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Internal pull requests

Add concise bullets under `Unreleased` for changes that affect SDK users: public
APIs, manifest formats, signing and verification, service integration, supported
Python versions and platforms, installation, and packaging requirements.
Internal refactors, tests, and typo fixes generally need no entry; explain the
omission in the PR description.

Use the relevant category, creating it only when needed:

| Category | Use for |
| --- | --- |
| Added | New capabilities |
| Changed | Updates to existing behavior |
| Deprecated | Features scheduled for removal |
| Removed | Features no longer available |
| Fixed | Corrected behavior |
| Security | Security fixes |

Include a PR link once available. Mark incompatible changes with **Breaking:**
and explain the required migration. For dependency updates worth documenting,
verify and state both versions (`from OLD to NEW`). Describe the effect on users
instead of copying commit messages.

Reviewers check that notes match the final implementation, use the right category,
and preserve other contributors' entries. Ordinary PRs do not assign release
versions or dates.

## Releases

Only plain `vX.Y.Z` tags get versioned changelog sections. Candidate and unofficial
tags (for example, `v2.3.1-rc1`) keep their changes under `Unreleased`; put
tag-specific notes in their GitHub releases. TestPyPI builds do not require a
stable release entry.

For an official stable release:

1. Review `Unreleased` against changes since the previous stable release. Move
   included changes into a new `## [X.Y.Z] - YYYY-MM-DD` section using the actual
   release date. Keep `Unreleased` at the top and releases newest first; omit
   empty category headings.
2. Match the heading to the intended `vX.Y.Z` tag. The release workflow sets the
   Python package version in `pyproject.toml` from the tag or manual version
   input; leave the source placeholder `0.0.0` in place.
3. Add a comparison link from the preceding stable tag to the new tag and advance
   the `Unreleased` link to compare the new tag with `main`. For example, for a
   hypothetical `v2.3.1` release following `v2.3.0`:

   ```markdown
   [Unreleased]: https://github.com/eqtylab/integrity-py/compare/v2.3.1...main
   [2.3.1]: https://github.com/eqtylab/integrity-py/compare/v2.3.0...v2.3.1
   ```

4. Run `./scripts/check-changelog.sh vX.Y.Z`, then merge the release preparation
   PR and follow the [release steps](README.md#release-steps). The
   [release workflow](.github/workflows/release.yml) checks for the matching
   `## [X.Y.Z]` heading before any builds or publication. Manual runs check the
   requested version too, accepting `X.Y.Z` or `vX.Y.Z`; select a ref containing
   the matching changelog entry and intended release code. Candidate and unofficial
   versions skip this check. Reviewers verify dates, notes, ordering, and links.

Correct factual errors in published entries when needed, but retain history.
If a release is withdrawn, append `[YANKED]` to its heading and explain why.
