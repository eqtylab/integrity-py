import json
from dataclasses import dataclass
from typing import Any, Dict

from eqty_sdk.core import get_cid_for_bytes


@dataclass
class StorageRecord:
    def __init__(self, type_: str, provider: Any, reference: Any):
        self.type = type_
        self.provider = provider
        self.reference = reference

    def cid(self) -> str:
        storage_json = json.dumps(self.to_dict())
        return get_cid_for_bytes(storage_json.encode("utf-8"))

    def to_dict(self) -> Dict[str, Any]:
        result: Dict[str, Any] = {}
        result["type"] = self.type
        result["provider"] = self.provider
        result["reference"] = self.reference

        return result
