from pathlib import Path
from uuid import UUID

from eqty_sdk import CID, Computation, Context, Dataset, Model, Signer, init, set_active_signer

parent_project = UUID("00000000-0000-0000-0000-000000000000")

# Recreate the same default subcontext under the Governance Studio project.
gov_ctx = Context.from_uuid(parent_project)
project_ctx = Context.with_parent(gov_ctx).new("Nested Contexts Example")
cfg = init(default_context=project_ctx)

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

run_ctx.export(Path("run-ctx.json"))
