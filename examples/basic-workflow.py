from pathlib import Path

from eqty_sdk import (
    Dataset,
    Signer,
    compute,
    init,
    set_active_signer,
)

cfg = init()

signer = Signer.new(name="Basic Workflow Signer")
set_active_signer(signer)


@compute(
    metadata={
        "name": "Prompt Formatter",
        "description": "Combine prompt inputs and a temperature setting into a single output.",
        "output_type": "dataset",
    }
)
def build_prompt(system_prompt: Dataset, user_prompt: str, temperature: float) -> str:
    return f"{system_prompt}\n\nUser: {user_prompt}\nTemperature: {temperature}"


system_prompt = Dataset.from_object(
    "You are a helpful assistant.",
    name="System Prompt",
    description="Base system prompt for the workflow",
)
user_prompt = "Summarize the last meeting in three bullets."

result = build_prompt(system_prompt, user_prompt, 0.2)
print(result)

ctx = cfg.get_default_context()
ctx.export(Path("basic-workflow.json"))
