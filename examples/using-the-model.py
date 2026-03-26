from pathlib import Path
from uuid import UUID

from eqty_sdk import CID, Computation, Context, Dataset, Model, Signer, init, set_active_signer

# UUID from `creating-the-model.py` actual UUID will vary
model_context_uuid = UUID("fea03473-4614-464f-a2de-3cbfdef603bb")

# Recreate the same default subcontext under the Governance Studio project.
model_ctx = Context.from_uuid(model_context_uuid)
ctx = Context.with_parent(model_ctx).new("Using the Model")
cfg = init(default_context=ctx)

signer = Signer.new(name="Nested Contexts Signer", _load_if_exists=True)
set_active_signer(signer)

# Reference a higher-level model asset from the parent context by CID.
shared_model = Model.from_cid(
    CID("bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"),
    name="Meeting Summarizer",
)

# During execution, create a child run context under the default context.
run_ctx = Context.with_parent(cfg.get_default_context()).new("run-2026-03-26")

input_data = Dataset.with_context(run_ctx).from_object(
    {"system_prompt": "You are a helpful assistant.", "user_prompt": "Summarize the meeting."},
    name="Prompt Inputs",
)
output_data = Dataset.with_context(run_ctx).from_object(
    {"summary": ["Decisions made", "Follow-ups assigned", "Next milestone set"]},
    name="Meeting Summary",
)

Computation.with_context(run_ctx).new(name="Summarize Meeting").add_input_cid(
    [shared_model.cid, input_data.cid]
).add_output_cid([output_data.cid]).finalize()

run_ctx.export(Path("./manifests/run-ctx.json"))
