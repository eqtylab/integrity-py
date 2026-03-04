import json
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

from eqty_sdk._rust import CID

from .statements import add_metadata_statement


class MetadataJSONEncoder(json.JSONEncoder):
    def default(self, o: Any) -> Any:
        if isinstance(o, Metadata):
            return o.to_dict()
        return super().default(o)


@dataclass
class Metadata:
    _additional_metadata: dict = field(default_factory=dict)

    def __init__(self, **kwargs):
        """Metadata is a data class that represents metadata that can be registered with the integirty platform.
        This metadata is used to describe any subject cid.
        Any additional metadata can be passed as keyword arguments.
        """
        self._additional_metadata = {}  # Initialize directly
        self._additional_metadata.update(kwargs)
        self._additional_metadata.pop("skip_proof", None)  # remove so it doesn't appear in metadata

    def __getattr__(self, attr):
        """Fallback to get additional metadata from the _additional_metadata dict."""
        return self._additional_metadata.get(attr)

    def to_dict(self) -> Dict[str, Any]:
        """Convert the Metadata instance to a dictionary."""
        metadata_dict = {}

        # Add additional metadata to the dictionary
        metadata_dict.update(self._additional_metadata)

        return metadata_dict

    def to_json_str(self) -> str:
        return json.dumps(self, indent=4, cls=MetadataJSONEncoder)

    def create_statement(self, subject_cid: CID, skip_proof: Optional[bool]) -> List[CID]:
        metadata_json = self.to_json_str()
        return add_metadata_statement(str(subject_cid), metadata_json, skip_proof=skip_proof)

    def __iter__(self):
        """Return an iterator that includes both the standard and additional metadata keys."""
        yield from self.to_dict().keys()

    def __contains__(self, key) -> bool:
        """Check if the key exists in the standard or additional metadata."""
        return key in self.to_dict()

    def __getitem__(self, key):
        """Allow dict-like access to metadata fields."""
        if key in self._additional_metadata:
            return self._additional_metadata[key]
        elif key in self.__dict__:
            return self.__dict__[key]
        else:
            raise KeyError(f"'{self.__class__.__name__}' object has no attribute '{key}'")
