# Verification

These functions check evidence you have been given, rather than producing any. They are the Python
counterpart of the checks the Integrity explorer runs in the browser, and they answer two separate
questions:

- **Is this statement still what it says it is?** Every lineage statement is identified by a hash of
  its own canonicalized content, so recomputing that hash and comparing it with the statement's `@id`
  detects any modification after the fact.
- **Is this credential's signature good, and is it about the statement I think it is?** A valid
  signature over some *other* subject proves nothing about the statement in hand, so the two checks
  belong together.

Both run **fully offline**. JSON-LD contexts resolve against documents compiled into the package, and
credentials are restricted to DID methods that can be resolved from the identifier itself, so
verification never reaches the network. This is what makes them usable in an air-gapped or
reproducible pipeline.

There is no manifest-level entry point. Verifying a whole manifest means looping these two functions
over its `statements`, which is what the explorer does:

```python
import json
from eqty_sdk import verify_statement, verify_vc

manifest = json.loads(open("manifest.json").read())
contexts = manifest.get("contexts")

for statement in manifest["statements"].values():
    if not verify_statement(json.dumps(statement), contexts):
        print("modified since it was created:", statement["@id"])
```

## `verify_statement`

Pass the manifest's `contexts` map whenever you have one. Statements reference their context by CID
(`"@context": "urn:cid:..."`), and while the common contexts ship inside the package, a statement
written against a custom context cannot be canonicalized without it.

A `True` result means every field the statement's `@context` defines is unmodified. It does not mean
the bytes are unmodified: the identifier commits to the statement's canonicalized RDF, and JSON-LD
expansion drops keys the context does not define.

::: eqty_sdk.verify_statement
    options:
      show_root_heading: true

## `verify_vc`

Supply `statement_id` whenever you know which statement the credential is supposed to attest. Without
it, only the signature is checked.

This verifies the cryptographic proof alone. Whether a credential has since been **revoked or
suspended** lives in its status list, which has to be fetched over the network and is deliberately
not consulted here.

::: eqty_sdk.verify_vc
    options:
      show_root_heading: true
