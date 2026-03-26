from pathlib import Path
from uuid import UUID

from eqty_sdk import Context, Model, Signer, compute, init, set_active_signer
from eqty_sdk.asset import Asset, Attribution, Config, Dataset

gov_studio_project = UUID("00000000-0000-0000-0000-000000000000")

# Create a local subcontext under the Governance Studio project and make it the default.
project_ctx = Context.from_uuid(gov_studio_project)
model_ctx = Context.with_parent(project_ctx).new("Model Context")
print(model_ctx)
cfg = init(default_context=model_ctx)

signer = Signer.new(name="Nested Contexts Signer", _load_if_exists=True)
set_active_signer(signer)


@compute(
    metadata={"description": "Creates a simple llm model"},
)
def create_model(model_name: Asset, provider: Asset, version: Asset) -> Model:
    return Model.from_object(
        {
            "model_name": model_name.value,
            "provider": provider.value,
            "version": version.value,
        },
        name=model_name.value,
        description=f"{provider.value} {version.value}",
    )


name = Config.from_object("Meeting Summarizer", name="Model Name")
provider = Dataset.from_object("Eqty", name="Model Provider")
version = Attribution.from_object("v1", name="Version")
create_model(name, provider, version)

cfg.get_default_context().export(Path("./manifests/default-ctx.json"))
