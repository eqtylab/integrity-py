# Changelog

Notable changes to integrity-py (`eqty_sdk`) are recorded here, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
See [CONTRIBUTING.md](CONTRIBUTING.md) for PR and release instructions.

This changelog covers stable releases starting at `v2.0.0`, the earliest stable
tag in this repository, with plain `vX.Y.Z` tags. Candidate,
unofficial, and development builds have no separate entries; upcoming changes
stay under `Unreleased`.

Entries predating 2026-09-10 were backfilled from Git history. Historical dates
are the tagged commits' committer dates, not verified package publication dates.
The first release summarizes the SDK at that tag; subsequent releases describe
changes since the preceding stable tag.

## [Unreleased]

### Added

- A changelog and contributor instructions for documenting PRs and releases.
- A CI check requiring a matching changelog heading before stable release builds
  and PyPI publication, including manual releases.

- Offline `verify_statement()` and `verify_vc()` APIs for checking statement
  integrity and credential signatures, including caller-supplied JSON-LD contexts.
  ([#71](https://github.com/eqtylab/integrity-py/pull/71),
  [#72](https://github.com/eqtylab/integrity-py/pull/72))
- Optional `delete_statements` and `delete_blobs` flags on `Context.register()`
  to clean up local data after successful service registration.
  ([#81](https://github.com/eqtylab/integrity-py/pull/81))
- A quick-start guide and expanded asset and context cleanup documentation.
  ([#74](https://github.com/eqtylab/integrity-py/pull/74),
  [#80](https://github.com/eqtylab/integrity-py/pull/80))

### Changed

- License the SDK under Apache-2.0 and move package publication from the private
  index to public PyPI, with Windows wheels and source distributions. Install
  with `pip install eqty-sdk` without private-index credentials.
  ([#74](https://github.com/eqtylab/integrity-py/pull/74),
  [#77](https://github.com/eqtylab/integrity-py/pull/77),
  [#78](https://github.com/eqtylab/integrity-py/pull/78),
  [#79](https://github.com/eqtylab/integrity-py/pull/79))
- Update the `integrity` dependency from revision `042e6609483e314c31e513bfe948f86f9b6d68a2`
  to `v0.0.13` for caller-supplied verification contexts.
  ([#72](https://github.com/eqtylab/integrity-py/pull/72))
- Group statement inserts into a single SQLite transaction and add indexes to
  speed up statement lookup.
  ([#73](https://github.com/eqtylab/integrity-py/pull/73))

### Fixed

- Include referenced governance declarations when generating manifests.
  ([#73](https://github.com/eqtylab/integrity-py/pull/73))
- Honor `_store` for computation inputs and outputs, including assets generated
  by the `@compute` decorator.
  ([#74](https://github.com/eqtylab/integrity-py/pull/74))

## [2.3.0] - 2026-08-06

### Added

- `Signer.load()` and `Signer.load_or_create()` for reusing named signers across
  sessions.
  ([#67](https://github.com/eqtylab/integrity-py/pull/67))

### Changed

- Update the `integrity` dependency from `v0.0.7` to revision
  `042e6609483e314c31e513bfe948f86f9b6d68a2` for the updated notary signer data.
  ([#64](https://github.com/eqtylab/integrity-py/pull/64),
  [#70](https://github.com/eqtylab/integrity-py/pull/70))

### Fixed

- Include the active VComp notary signer's credentials and DID blobs in exported
  manifests.
  ([#70](https://github.com/eqtylab/integrity-py/pull/70))
- Create missing parent directories when exporting a manifest to a file.
  ([#66](https://github.com/eqtylab/integrity-py/pull/66))

## [2.2.0] - 2026-06-23

### Changed

- Upgrade the `integrity` dependency from `v0.0.1` to `v0.0.7` and adapt credential
  creation to the updated API.
  ([#63](https://github.com/eqtylab/integrity-py/pull/63))

### Security

- Update the locked Rust `openssl` dependency from `0.10.78` to `0.10.79`,
  `openssl-sys` from `0.9.114` to `0.9.115`, and `rustls-webpki` from `0.103.9`
  to `0.103.13` to address reported vulnerabilities.
  ([#61](https://github.com/eqtylab/integrity-py/pull/61))
- Raise the minimum `urllib3` version from `2.6.3` to `2.7.0` for security fixes.
  ([#62](https://github.com/eqtylab/integrity-py/pull/62))

## [2.1.2] - 2026-05-05

### Fixed

- Retrieve DID metadata whose subject is either the DID itself or its DID
  registration statement ID.
  ([#60](https://github.com/eqtylab/integrity-py/pull/60))

## [2.1.1] - 2026-05-04

### Fixed

- Build Linux wheels against an explicitly built OpenSSL 3.0.17 to avoid
  incompatible distribution-provided OpenSSL versions.
  ([#59](https://github.com/eqtylab/integrity-py/pull/59))

### Removed

- Stop publishing source distributions after uploads exceeded the package
  server's size limit; use a wheel or build from a source checkout.
  ([#59](https://github.com/eqtylab/integrity-py/pull/59))

## [2.1.0] - 2026-05-01

### Added

- A `Credential` asset type and top-level exports for additional supported
  asset classes, including `Binary`, `Guardrail`, `Prompt`, and `Tool`.
  ([#56](https://github.com/eqtylab/integrity-py/pull/56))

### Changed

- **Breaking:** Align built-in asset types with Governance Studio. Replace
  `Config` / `AssetType.CONFIG` with `Configuration` / `AssetType.CONFIGURATION`.
  `Attribution` / `AssetType.ATTRIBUTION` are removed; use a supported asset type
  or `Custom` for application-specific types.
  ([#56](https://github.com/eqtylab/integrity-py/pull/56))

### Fixed

- Import VComp statements into the default context so their graph ID exists.
  ([#57](https://github.com/eqtylab/integrity-py/pull/57))

## [2.0.9] - 2026-04-10

### Added

- Publish musllinux wheels for x86_64 and aarch64 alongside manylinux wheels,
  enabling installation on musl-based systems such as Alpine Linux.
  ([#52](https://github.com/eqtylab/integrity-py/pull/52))

## [2.0.8] - 2026-04-08

### Added

- macOS x86_64 wheels in addition to Apple Silicon wheels.
  ([#51](https://github.com/eqtylab/integrity-py/pull/51))
- Expanded workflow examples and installation, API, and model-signing guides.
  ([#47](https://github.com/eqtylab/integrity-py/pull/47))

### Fixed

- Apply the computation's context to assets automatically created by `@compute`.
  ([#51](https://github.com/eqtylab/integrity-py/pull/51))
- Persist VComp signer blobs and statements when creating or reloading a signer,
  so evidence is available for manifest generation.
  ([#50](https://github.com/eqtylab/integrity-py/pull/50))
- Correct source-distribution build permissions during release publication.
  ([#51](https://github.com/eqtylab/integrity-py/pull/51))

## [2.0.7] - 2026-03-25

### Added

- `Prompt` and `SystemPrompt` asset types.
  ([#41](https://github.com/eqtylab/integrity-py/pull/41))
- Versioned documentation with `dev`, `latest`, and numbered releases, wheel
  runtime requirement reports, and examples for context linking, service
  registration, and model signing.
  ([#45](https://github.com/eqtylab/integrity-py/pull/45))

### Removed

- **Breaking:** Remove `Signer.yubihsm2()` and YubiHSM support. Applications using
  it must select a supported local, auth-service, or VComp notary signer.
  ([#46](https://github.com/eqtylab/integrity-py/pull/46))

## [2.0.6] - 2026-03-24

### Fixed

- Resolve VComp signer DID information and include related statements in
  manifests.
  ([#43](https://github.com/eqtylab/integrity-py/pull/43))
- Index model-signing statements by subject and include their Sigstore evidence
  when the subject CID is referenced by a manifest.
  ([#44](https://github.com/eqtylab/integrity-py/pull/44))

## [2.0.5] - 2026-03-23

### Added

- Reuse persisted signers by passing a name and `_load_if_exists=True` to signer
  factories.
  ([#39](https://github.com/eqtylab/integrity-py/pull/39))

### Changed

- **Breaking:** Raise the declared minimum Python version from 3.9 to 3.10;
  upgrade Python 3.9 environments before installing. Use Python 3.10 instead of
  3.12 for builds and manylinux2014 containers for broader Linux compatibility.
  ([#39](https://github.com/eqtylab/integrity-py/pull/39))

## [2.0.4] - 2026-03-20

### Added

- `Config`, `Skill`, and `Tool` asset types and an MkDocs API reference.
  ([#35](https://github.com/eqtylab/integrity-py/pull/35))

### Changed

- **Breaking:** Prefix control keyword arguments with underscores. Replace
  `store=` with `_store=` and `skip_proof=` with `_skip_proof=` in asset,
  computation, CID, and statement calls where applicable.
  ([#35](https://github.com/eqtylab/integrity-py/pull/35))
- **Breaking:** Model-signing signatures are opt-in. Use
  `Model.from_path(directory, _enable_model_signing_signature=True)` with a
  compatible SECP256R1 signer to request Sigstore evidence.
  ([#35](https://github.com/eqtylab/integrity-py/pull/35),
  [#36](https://github.com/eqtylab/integrity-py/pull/36))
- **Breaking:** Rename `Context.from_parent()` to `Context.with_parent()`.
  `Context.from_uuid()` now returns the context with that UUID directly;
  create children through `Context.with_parent(context).new(name)`.
  ([#35](https://github.com/eqtylab/integrity-py/pull/35))
- **Breaking:** Remove `DID.with_context()` and `DidFactory`; create DIDs directly
  with `DID.from_signer()` or `DID.from_did_string()`.
  ([#35](https://github.com/eqtylab/integrity-py/pull/35))

### Fixed

- Preserve typed asset subclasses when constructing assets, improving Python
  type checking and editor support.
  ([#35](https://github.com/eqtylab/integrity-py/pull/35))

## [2.0.3] - 2026-03-18

### Fixed

- Include statements referenced through associations in manifest queries.
  ([#32](https://github.com/eqtylab/integrity-py/pull/32))
- Improve generated Python type stubs and top-level exports for language-server
  support.
  ([#32](https://github.com/eqtylab/integrity-py/pull/32))

## [2.0.2] - 2026-03-17

### Added

- `Context.import_manifest()` to import existing manifests into the local index.
  ([#29](https://github.com/eqtylab/integrity-py/pull/29))

## [2.0.1] - 2026-03-11

### Changed

- **Breaking:** Make `default_context` the first positional argument to `init()`
  and make `custom_dir` keyword-only. Update calls to
  `init(context, custom_dir=path)` or use explicit keywords.
  ([#28](https://github.com/eqtylab/integrity-py/pull/28))

## [2.0.0] - 2026-03-10

Initial stable tag in this repository, establishing the Rust-backed Python SDK.

### Added

- Typed assets, content identifiers, computation tracking, and SQLite-backed
  contexts for collecting lineage statements and exporting manifests.
  ([#3](https://github.com/eqtylab/integrity-py/pull/3),
  [#7](https://github.com/eqtylab/integrity-py/pull/7),
  [#14](https://github.com/eqtylab/integrity-py/pull/14))
- Local and service-backed signers, DID statement retrieval, metadata and
  governance statements, and model signing with Sigstore evidence.
  ([#8](https://github.com/eqtylab/integrity-py/pull/8),
  [#12](https://github.com/eqtylab/integrity-py/pull/12),
  [#20](https://github.com/eqtylab/integrity-py/pull/20))
- Batch upload support for registering manifests with the Integrity service.
  ([#25](https://github.com/eqtylab/integrity-py/pull/25))
- Generated API documentation and release packaging for Linux and macOS.
  ([#16](https://github.com/eqtylab/integrity-py/pull/16),
  [#26](https://github.com/eqtylab/integrity-py/pull/26),
  [#27](https://github.com/eqtylab/integrity-py/pull/27))

[Unreleased]: https://github.com/eqtylab/integrity-py/compare/v2.3.0...main
[2.3.0]: https://github.com/eqtylab/integrity-py/compare/v2.2.0...v2.3.0
[2.2.0]: https://github.com/eqtylab/integrity-py/compare/v2.1.2...v2.2.0
[2.1.2]: https://github.com/eqtylab/integrity-py/compare/v2.1.1...v2.1.2
[2.1.1]: https://github.com/eqtylab/integrity-py/compare/v2.1.0...v2.1.1
[2.1.0]: https://github.com/eqtylab/integrity-py/compare/v2.0.9...v2.1.0
[2.0.9]: https://github.com/eqtylab/integrity-py/compare/v2.0.8...v2.0.9
[2.0.8]: https://github.com/eqtylab/integrity-py/compare/v2.0.7...v2.0.8
[2.0.7]: https://github.com/eqtylab/integrity-py/compare/v2.0.6...v2.0.7
[2.0.6]: https://github.com/eqtylab/integrity-py/compare/v2.0.5...v2.0.6
[2.0.5]: https://github.com/eqtylab/integrity-py/compare/v2.0.4...v2.0.5
[2.0.4]: https://github.com/eqtylab/integrity-py/compare/v2.0.3...v2.0.4
[2.0.3]: https://github.com/eqtylab/integrity-py/compare/v2.0.2...v2.0.3
[2.0.2]: https://github.com/eqtylab/integrity-py/compare/v2.0.1...v2.0.2
[2.0.1]: https://github.com/eqtylab/integrity-py/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/eqtylab/integrity-py/tree/v2.0.0
