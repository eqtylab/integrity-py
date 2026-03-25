from pathlib import Path

from eqty_sdk import Computation, Context, Dataset, Signer, init, set_active_signer

root_ctx = Context.new("customer-project")
run_ctx = Context.with_parent(root_ctx).new("daily-run-2026-03-25")

cfg = init(default_context=root_ctx)
signer = Signer.new(name="Context Linking Signer")
set_active_signer(signer)

input_data = Dataset.with_context(run_ctx).from_object(
    {"rows": 128, "source": "warehouse.snapshot"},
    name="Input Batch",
)
output_data = Dataset.with_context(run_ctx).from_object(
    {"summary": "cleaned and normalized"},
    name="Normalized Batch",
)

Computation.with_context(run_ctx).new(name="Normalize Batch").add_input_cid(
    [input_data.cid]
).add_output_cid([output_data.cid]).finalize()

cfg.get_default_context().export(Path("customer-project.json"))
run_ctx.export(Path("daily-run-2026-03-25.json"))
