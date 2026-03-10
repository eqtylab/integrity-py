import json
import logging
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

from eqty_sdk._rust import CID, get_cid_for_json, signer

logger = logging.getLogger("eqty.sdk.declaration")


@dataclass
class Declaration:
    subject_line: str
    statement: str

    submitted_at: Optional[str] = None
    submitted_by: Optional[str] = None
    control_cid: List[str] = field(default_factory=list)
    attachment_cid: List[str] = field(default_factory=list)
    extra: Dict[str, str] = field(default_factory=dict)
    _cid: Optional[CID] = field(default=None, init=False, repr=False)

    def __init__(self, subject_line: str, statement: str):
        self.subject_line = subject_line
        self.statement = statement
        self.submitted_by = None
        self.attachment_cid = []
        self.control_cid = []
        self.extra = {}
        self._cid = None

    @staticmethod
    def new(subject_line: str, statement: str) -> "Declaration":
        declaration = Declaration(subject_line, statement)
        return declaration

    def add_attachment_cid(self, cid: str) -> "Declaration":
        """Appends the CID to the attachment cid list of the declaration."""
        self.attachment_cid.append(cid)
        return self

    def add_control_cid(self, cid: str) -> "Declaration":
        """Appends the CID to the control cid list of the declaration."""
        self.control_cid.append(cid)
        return self

    def add_extra(self, key: str, val: str) -> "Declaration":
        """Adds or Overwrites the key/value in the extra field of the declaration."""
        self.extra[key] = val
        return self

    def finalize(self) -> "Declaration":
        self.submitted_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        self.submitted_by = signer.get_active_signer_did_key()

        declaration_json = self.to_json()
        logger.debug(f"Finalizing declaration. {declaration_json}")
        self._cid = get_cid_for_json(declaration_json, True)
        logger.info(f"Declaration CID: {self._cid}")
        return self

    def cid(self) -> CID:
        if self._cid is not None:
            return self._cid
        raise RuntimeError("you must call .finalize() before you can access the declarations cid.")

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary, omitting empty/None values."""
        result: Dict[str, Any] = {}

        result["submittedAt"] = self.submitted_at
        result["submittedBy"] = self.submitted_by
        result["controlCid"] = self.control_cid

        # Add optional fields only if they have values
        if self.subject_line:
            result["subjectLine"] = self.subject_line
        if self.statement:
            result["statement"] = self.statement
        if self.attachment_cid:
            result["attachmentCid"] = self.attachment_cid
        if self.extra:
            result["extra"] = self.extra

        return result

    def to_json(self) -> str:
        """Convert to JSON string."""
        return json.dumps(self.to_dict())
