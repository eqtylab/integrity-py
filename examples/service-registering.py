from uuid import UUID

from eqty_sdk import (
    Agent,
    Computation,
    Context,
    Dataset,
    Model,
    Service,
    Signer,
    init,
    set_active_signer,
)

gov_studio_project = UUID("00000000-0000-0000-0000-000000000000")
ctx = Context.from_uuid(gov_studio_project).new("My Agent v1")
cfg = init(default_context=ctx)

signer = Signer.new()
set_active_signer(signer)

user_prompt = Dataset.from_object("User Prompt", name="User Prompt")
system_prompt = Dataset.from_object("System Prompt", name="System Prompt")
model_v1 = Model.from_object("GPT-4", name="GPT-4")
agent = Agent.from_object("Agent v1", name="Agent 1")

Computation.new(name="Agent 1 Compute").add_input_cid(
    [user_prompt.cid, system_prompt.cid, model_v1.cid, agent.cid]
).add_output_cid([Dataset.from_object("Output1", name="Output 1").cid]).finalize()

service = Service.new("http://localhost:3050")

cfg.get_default_context().register(service)
