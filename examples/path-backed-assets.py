from pathlib import Path

from eqty_sdk import Code, Database, Dataset, Document, Signer, init, set_active_signer

cfg = init()

signer = Signer.new(name="Path Asset Signer")
set_active_signer(signer)

repo_root = Path(__file__).resolve().parents[1]
fixture_path = repo_root / "tests/fixtures/assets/datasets/file/file_text.txt"
source_path = repo_root / "examples/path-backed-assets.py"

dataset = Dataset.from_path(fixture_path, name="Fixture Dataset")
document = Document.from_path(fixture_path, name="Fixture Document")
database = Database.from_path(fixture_path, name="Fixture Database")
code = Code.from_path(source_path, name="Example Source")

print(dataset.cid)
print(document.cid)
print(database.cid)
print(code.cid)

cfg.get_default_context().export(Path("./manifests/path-backed-assets.json"))
