# This file is used in the CI pipeline to test that the sdk is installed properly.
from importlib.metadata import version

print(f"Imported version '{version('eqty-sdk')}' of the sdk")
