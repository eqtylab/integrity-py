# Re-export Entity and creation functions from Rust module
from typing import TYPE_CHECKING

from eqty_sdk._rust import entity as _entity_module

if TYPE_CHECKING:
    from eqty_sdk._rust import Entity as Entity
else:
    Entity = _entity_module.Entity

# Re-export the creation functions
create_entity = _entity_module.create_entity
create_entity_from_uuid = _entity_module.create_entity_from_uuid

__all__ = ["Entity", "create_entity", "create_entity_from_uuid"]
