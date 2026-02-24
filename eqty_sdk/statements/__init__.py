from eqty_sdk._rust import Statements, statements as eqty_core_statements

add_data_statement = eqty_core_statements.add_data_statement
add_association_statement = eqty_core_statements.add_association_statement
add_computation_statement = eqty_core_statements.add_computation_statement
add_did_statement = eqty_core_statements.add_did_statement
add_entity_statement = eqty_core_statements.add_entity_statement
add_metadata_statement = eqty_core_statements.add_metadata_statement
add_storage_statement = eqty_core_statements.add_storage_statement
add_governance_statement = eqty_core_statements.add_governance_statement
add_vc_statement = eqty_core_statements.add_vc_statement

__all__ = [
    "add_association_statement",
    "add_computation_statement",
    "add_data_statement",
    "add_did_statement",
    "add_entity_statement",
    "add_governance_statement",
    "add_metadata_statement",
    "add_storage_statement",
    "add_vc_statement",
    "Statements",
]
