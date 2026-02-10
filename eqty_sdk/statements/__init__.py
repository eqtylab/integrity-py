from eqty_sdk._rust import statements as eqty_core_statements
from .common import Statements
from .computation import add_computation_statement
from .did import add_did_statement
from .entity import add_entity_statement
from .governance import add_governance_statement
from .metadata import add_metadata_statement
from .storage import add_storage_statement

add_data_statement = eqty_core_statements.add_data_statement
add_association_statement = eqty_core_statements.add_association_statement

__all__ = [
    "add_association_statement",
    "add_computation_statement",
    "add_data_statement",
    "add_did_statement",
    "add_entity_statement",
    "add_governance_statement",
    "add_metadata_statement",
    "add_storage_statement",
    "Statements",
]
