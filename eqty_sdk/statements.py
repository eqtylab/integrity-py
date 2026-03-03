import os
from typing import List, Optional, Union

from eqty_sdk._rust import (
    CID,
    DID,
    UUID,
    Graph as Context,
    PyAssociationType,
    statements as eqty_core_statements,
)

add_data_statement = eqty_core_statements.add_data_statement
add_association_statement = eqty_core_statements.add_association_statement
add_computation_statement = eqty_core_statements.add_computation_statement
add_did_statement = eqty_core_statements.add_did_statement
add_entity_statement = eqty_core_statements.add_entity_statement
add_metadata_statement = eqty_core_statements.add_metadata_statement
add_storage_statement = eqty_core_statements.add_storage_statement
add_governance_statement = eqty_core_statements.add_governance_statement
add_vc_statement = eqty_core_statements.add_vc_statement


class _AssociationTypes:
    CERTIFIES = PyAssociationType.Certifies
    INCLUDES = PyAssociationType.Includes
    IS_INSTANCE_OF = PyAssociationType.IsInstanceOf


ASSOCIATION_TYPES = _AssociationTypes


def _normalize_association_ref(value: object) -> str:
    if isinstance(value, CID):
        return str(value)
    if isinstance(value, UUID):
        return str(value)
    if isinstance(value, DID):
        did_value = getattr(value, "did", None)
        if isinstance(did_value, str) and did_value:
            return did_value
        raise ValueError("DID instance does not expose its DID string.")
    raise ValueError("Invalid association reference type.")


class Association:
    def __init__(self):
        raise TypeError("Use Association.new() to create instances of this class.")

    def __init_internal__(
        self,
        ctx: Optional[Context],
        subject: object,
        association_type: PyAssociationType,
        skip_proof: Optional[bool] = None,
    ):
        self._ctx = ctx
        self._subject = _normalize_association_ref(subject)
        self._association_type = association_type
        self._predicates: List[str] = []

        if skip_proof is not None:
            self._skip_proof = skip_proof
        else:
            self._skip_proof = os.getenv("EQTY_SKIP_PROOF", "").lower() == "true"

    @classmethod
    def new(cls, subject: object, association_type: PyAssociationType, **kwargs) -> "Association":
        skip_proof = kwargs.pop("skip_proof", None)
        instance = object.__new__(cls)
        instance.__init_internal__(None, subject, association_type, skip_proof)
        return instance

    @staticmethod
    def with_context(ctx: Context):
        class _Factory:
            def new(
                self, subject: object, association_type: PyAssociationType, **kwargs
            ) -> "Association":
                skip_proof = kwargs.pop("skip_proof", None)
                instance = object.__new__(Association)
                instance.__init_internal__(ctx, subject, association_type, skip_proof)
                return instance

        return _Factory()

    def add_predicate(self, predicate: Union[object, List[object]]) -> "Association":
        if isinstance(predicate, list):
            self._predicates.extend(_normalize_association_ref(item) for item in predicate)
        else:
            self._predicates.append(_normalize_association_ref(predicate))
        return self

    def finalize(self) -> "Association":
        add_association_statement(
            self._subject,
            self._predicates,
            self._association_type,
            skip_proof=self._skip_proof,
            graph=self._ctx,
        )
        return self


__all__ = [
    "ASSOCIATION_TYPES",
    "Association",
    "add_association_statement",
    "add_computation_statement",
    "add_data_statement",
    "add_did_statement",
    "add_entity_statement",
    "add_governance_statement",
    "add_metadata_statement",
    "add_storage_statement",
    "add_vc_statement",
]
