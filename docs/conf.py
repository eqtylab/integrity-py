import os
import sys

sys.path.insert(0, os.path.abspath(".."))

project = "eqty_sdk"
author = "Eqty"

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
    "sphinx.ext.viewcode",
    "sphinx_autodoc_typehints",
    "myst_parser",
]

templates_path = ["_templates"]
exclude_patterns = ["_build"]

root_doc = "index"

autodoc_default_options = {
    "members": True,
    "undoc-members": False,
    "show-inheritance": True,
}

autodoc_typehints = "description"
