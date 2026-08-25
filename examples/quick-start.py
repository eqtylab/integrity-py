from pathlib import Path

from eqty_sdk import (
    CID,
    DID,
    SIGNER_ALGORITHMS,
    Computation,
    Dataset,
    Signer,
    compute,
    init,
    set_active_signer,
)

# Configure SDK
ctx = init().set_store_all_blobs(True).get_default_context()

# Set signer
signer = Signer.new(SIGNER_ALGORITHMS.SECP256R1)
set_active_signer(signer)
did = DID.from_signer(
    signer, name="My key", description="My Ed25519 signing key for integrity statements."
)

# Create sample objects
my_object = "My Object"
my_path = "./"
my_cid = CID("bafkr4icqw77khu73vgw74jpnlnep37ec3l6jd4lg5kvw2letvqjhgk6jmi")

# Registering a serializable Python object
d0 = Dataset.from_object(my_object, name="My dataset 0", description="My description for dataset 0")

# Registering a file or directory of files from the file system
d1 = Dataset.from_path(
    my_path,
    name="My dataset 1",
    description="My description for dataset 1",
)

# Registering a data asset or collection of assets by its CID
d2 = Dataset.from_cid(
    my_cid, name="My dataset 2", description="My description for dataset 2", foo="bar"
)

# Registering a computation with builder
computation = (
    Computation.new().add_input_cid(d0.cid).add_input_cid(d1.cid).add_output_cid(d2.cid).finalize()
)


@compute(
    metadata={
        "name": "My computation",
        "description": "My description for the computation",
        "foo": "bar",
    }
)
def my_function(input_0: Dataset, input_1: Dataset):
    my_output_object = str(input_0.value) + str(input_1.value)
    output = Dataset.from_object(
        my_output_object, name="My dataset", description="My description for the output dataset"
    )
    return output


my_function(d0, d1)

# Export manifest
path = Path("./manifest_simple.json")
ctx.export(path)
ctx.delete_tree()
## Optionally delete all the created statements
# purge_statement_store()

## Optionally delete all the stored blobs
# purge_blob_store()
