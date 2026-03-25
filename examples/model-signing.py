from pathlib import Path

from eqty_sdk import SIGNER_ALGORITHMS, Model, Signer, init, set_active_signer
from eqty_sdk.compute.computation import Computation

cfg = init()
repo_root = Path(__file__).resolve().parents[1]

signer = Signer.new(SIGNER_ALGORITHMS.SECP256R1)
set_active_signer(signer)
model = Model.from_path(
    repo_root / "tests/fixtures",
    name="SECP256R1 Model",
    _enable_model_signing_signature=True,
)

Computation.new().add_input_cid(model.cid).add_output_object("Output").finalize()

ctx = cfg.get_default_context()
ctx.export(Path("model-signing.json"))
