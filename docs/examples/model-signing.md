# Model Signing

Model signing is explicit. A model-signing statement is only attempted when:

- the asset is a model
- the source path is a directory
- `_enable_model_signing_signature=True`

```python
from eqty_sdk import init, Model, set_active_signer, Signer, SIGNER_ALGORITHMS

cfg = init()

ed_signer = Signer.new(SIGNER_ALGORITHMS.ED25519)
set_active_signer(ed_signer)
Model.from_path(
    "./examples/fixtures",
    name="ED25519 Model",
    _enable_model_signing_signature=True,
)

r1_signer = Signer.new(SIGNER_ALGORITHMS.SECP256R1)
set_active_signer(r1_signer)
Model.from_path(
    "./examples/fixtures",
    name="SECP256R1 Model",
    _enable_model_signing_signature=True,
)
```

If the active signer is not compatible with model signing, the SDK logs a warning instead of failing asset creation.
