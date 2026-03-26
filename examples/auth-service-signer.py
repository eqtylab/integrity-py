from pathlib import Path

from eqty_sdk import Dataset, Signer, init, set_active_signer

cfg = init()

signer = Signer.auth_service(
    "http://localhost:3050",
    name="gov-studio-signer",
    _load_if_exists=True,
)
set_active_signer(signer)

payload = Dataset.from_object(
    {"message": "Signed with an auth-service signer"},
    name="Remote Signed Payload",
)

print(payload.cid)
cfg.get_default_context().export(Path("./manifests/auth-service-signer.json"))
