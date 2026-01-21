from .association import add_association_statement
from .common import Statements
from .computation import add_computation_statement
from .data import add_data_statement
from .did import add_did_statement
from .entity import add_entity_statement
from .governance import add_governance_statement
from .metadata import add_metadata_statement
from .storage import add_storage_statement

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
