# Signer

Signers are the identity and signing mechanism used by the SDK when it needs to produce signed provenance statements, attestations, or model-signing metadata.

A signer is required, as every statement that gets created is attributed back to being created by a Signer's underlying DID (Decentralized Identifier). A signer is part of process setup: create or load a signer, call [`set_active_signer`](global-functions.md), and then let higher-level SDK operations use that active signer automatically.

The SDK supports local signers and service-backed signers. Choose the one that matches your security model and operational environment.

## Creating vs. loading

Signers are persisted to disk, so a script that runs twice must not try to generate the
same named signer twice.

| Call | Behaviour |
| --- | --- |
| `Signer.new(...)` | Always generates a new key. Raises `ValueError` if `name` is already taken. |
| `Signer.load(name)` | Loads a persisted signer. Raises `LookupError` if it does not exist. |
| `Signer.load_or_create(name)` | Generates on the first run, reuses on later runs. Idempotent. |

`Signer.load_or_create(...)` is the one to reach for in a script you expect to run more
than once — it keeps the DID stable across runs.

```python
signer = Signer.load_or_create(name="My Workflow Signer")
set_active_signer(signer)
```

## Signer

::: eqty_sdk._rust.Signer
    options:
      members_order: source
      show_root_toc_entry: false
      show_if_no_docstring: true

## Signer Algorithms

The algorithm enum controls which key type is created for signer flows that generate or import keys.

Most users can use the default unless they have an interoperability or policy reason to choose a specific algorithm.

::: eqty_sdk._rust.SIGNER_ALGORITHMS
    options:
      members:
        - ED25519
        - SECP256K1
        - SECP256R1
      members_order: source
      show_root_toc_entry: false
      show_if_no_docstring: true
