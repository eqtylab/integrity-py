from pathlib import Path

from eqty_sdk import Dataset, Signer, compute, init, set_active_signer

# Initialize local SDK state and retain content that is hashed in this workflow.
cfg = init()
cfg.set_store_all_blobs(True)

# A named local signer creates a software key on the first run and reuses it later.
# Its DID signs the provenance statements created by this process.
signer = Signer.load_or_create(name="Quick Start Signer")
set_active_signer(signer)


@compute(
    metadata={
        "name": "Prompt Formatter",
        "description": "Combine a system prompt, user prompt, and temperature.",
        "output_type": "dataset",
    }
)
def build_prompt(system_prompt: Dataset, user_prompt: str, temperature: float) -> str:
    return f"{system_prompt}\n\nUser: {user_prompt}\nTemperature: {temperature}"


system_prompt = Dataset.from_object(
    "You are a helpful assistant.",
    name="System Prompt",
)

result = build_prompt(
    system_prompt,
    "Summarize the last meeting in three bullets.",
    0.2,
)
print(result)

cfg.get_default_context().export(Path("./manifests/quick-start.json"))
