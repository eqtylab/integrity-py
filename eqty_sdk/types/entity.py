import json
from uuid import UUID, uuid4

from eqty_sdk.metadata import MetadataJSONEncoder
from eqty_sdk.statements import add_metadata_statement
from eqty_sdk.statements.entity import add_entity_statement


class Entity:
    """Represents an unhashed entity."""

    def __init__(self, uuid: UUID):
        self.uuid = uuid

    @staticmethod
    def new(**kwargs) -> "Entity":
        new_uuid = uuid4()
        statement_ids = add_entity_statement(str(new_uuid), None)
        __create_metadata_statement__(statement_ids[0], **kwargs)

        return Entity(new_uuid)

    @staticmethod
    def from_uuid(uuid: UUID, **kwargs) -> "Entity":
        statement_ids = add_entity_statement(str(uuid), None)
        __create_metadata_statement__(statement_ids[0], **kwargs)
        return Entity(uuid)


def __create_metadata_statement__(subject: str, **kwargs) -> None:
    metadata = json.dumps(kwargs, indent=4, cls=MetadataJSONEncoder)
    add_metadata_statement(subject, metadata)
