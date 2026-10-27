import logging
from datetime import datetime
from pathlib import Path
from typing import Any, List, Optional, Union, cast

from eqty_sdk._rust import (
    CID,
    Context,
    get_cid_for_bytes,
    get_cid_for_path,
)
from eqty_sdk.asset import serialize_for_hashing
from eqty_sdk.context import get_active_context
from eqty_sdk.errors import UsageError
from eqty_sdk.metadata import Metadata
from eqty_sdk.statements import add_computation_statement, otel_span_urn

logger = logging.getLogger("eqty.sdk.computation")


def __cid_path__(path: Union[str, Path], _store: Optional[bool] = None) -> CID:
    """Resolves the path, and then calculates the CID."""
    path2 = Path(path) if isinstance(path, str) else path
    resolved_path = path2.resolve()

    if resolved_path.is_dir():
        return get_cid_for_path(resolved_path, _store)
    else:
        try:
            return get_cid_for_path(resolved_path, _store)
        except (IOError, OSError):
            raise UsageError(f"The input path '{resolved_path}' was not found.")


def __rfc3339__(time: Union[datetime, str]) -> str:
    """Formats a timezone-aware datetime as RFC 3339; a string is passed through as given."""
    if isinstance(time, str):
        return time
    if time.tzinfo is None or time.utcoffset() is None:
        raise ValueError("Computation times must be timezone-aware.")
    return time.isoformat()


class Computation:
    def __init__(self):
        raise TypeError("Use Computation.new() to create instances of this class.")

    def __init_internal__(
        self,
        ctx: Optional[Context],
        metadata: Metadata,
        _skip_proof: Optional[bool] = None,
        _store: Optional[bool] = None,
    ):
        self._ctx = ctx
        self._metadata = metadata
        self._input_cids: List[CID] = []
        self._output_cids: List[CID] = []
        self._computation_cid: Union[CID, None] = None
        self._start_time: Optional[str] = None
        self._end_time: Optional[str] = None
        self._span: Optional[str] = None
        self._parent_span: Optional[str] = None
        self._statement_id: Optional[CID] = None
        self._skip_proof = _skip_proof
        self._store = _store

    @classmethod
    def new(cls, **kwargs) -> "Computation":
        _skip_proof = kwargs.pop("_skip_proof", None)
        _store = kwargs.pop("_store", None)
        ctx = get_active_context()

        metadata = Metadata(**kwargs)
        instance = object.__new__(cls)
        instance.__init_internal__(ctx, metadata, _skip_proof, _store)
        return instance

    @staticmethod
    def with_context(ctx: Context):
        class _Factory:
            def new(self, **kwargs) -> "Computation":
                _skip_proof = kwargs.pop("_skip_proof", None)
                _store = kwargs.pop("_store", None)

                metadata = Metadata(**kwargs)
                instance = object.__new__(Computation)
                instance.__init_internal__(ctx, metadata, _skip_proof, _store)
                return instance

        return _Factory()

    def add_input_cid(self, cid: Union[List[CID], CID]) -> "Computation":
        """Adds the CID(s) to the computations input list."""
        if isinstance(cid, CID):
            self._input_cids.append(cid)
        elif isinstance(cid, list) and all(isinstance(item, CID) for item in cid):
            cids = [cast(CID, item) for item in cid]
            self._input_cids.extend(cids)
        else:
            raise ValueError("Invalid type for cid")

        return self

    def add_input_path(self, path: Union[List[Path], List[str], Path, str]) -> "Computation":
        """Resolves the provide path(s) and adds the computed CID(s) to the computations input list."""
        if isinstance(path, Path):
            self._input_cids.append(__cid_path__(path, self._store))
        elif isinstance(path, list) and all(isinstance(item, Path) for item in path):
            self._input_cids.extend([__cid_path__(p, self._store) for p in path])
        elif isinstance(path, str):
            self._input_cids.append(__cid_path__(path, self._store))
        elif isinstance(path, list) and all(isinstance(item, str) for item in path):
            for p in path:
                self._input_cids.append(__cid_path__(p, self._store))
        else:
            raise ValueError("Invalid type for path")
        return self

    def add_input_object(self, obj: Union[List[Any], Any]) -> "Computation":
        """Serializes the obj(s), then calculates the CID for the serialized bytes. The CID(s) are appended to the computations input list."""
        if isinstance(obj, list):
            for o in obj:
                serialized_bytes = serialize_for_hashing(o)
                cid = get_cid_for_bytes(serialized_bytes, self._store)
                self._input_cids.append(cid)
        else:
            serialized_bytes = serialize_for_hashing(obj)
            cid = get_cid_for_bytes(serialized_bytes, self._store)
            self._input_cids.append(cid)

        return self

    def add_output_cid(self, cid: Union[List[CID], CID]) -> "Computation":
        """Adds the CID(s) to the computations output list."""
        if isinstance(cid, CID):
            self._output_cids.append(cid)
        elif isinstance(cid, list) and all(isinstance(item, CID) for item in cid):
            cids = [cast(CID, item) for item in cid]
            self._output_cids.extend(cids)
        else:
            raise ValueError("Invalid type for cid")

        return self

    def add_output_path(self, path: Union[List[Path], List[str], Path, str]) -> "Computation":
        """Resolves the provide path(s) and adds the computed CID(s) to the computations output list."""
        if isinstance(path, Path):
            self._output_cids.append(__cid_path__(path, self._store))
        elif isinstance(path, list) and all(isinstance(item, Path) for item in path):
            self._output_cids.extend([__cid_path__(p, self._store) for p in path])
        elif isinstance(path, str):
            self._output_cids.append(__cid_path__(path, self._store))
        elif isinstance(path, list) and all(isinstance(item, str) for item in path):
            for p in path:
                self._output_cids.append(__cid_path__(p, self._store))
        else:
            raise ValueError("Invalid type for path")
        return self

    def add_output_object(self, obj: Union[List[Any], Any]) -> "Computation":
        """Serializes the obj(s), then calculates the CID for the serialized bytes. The CID(s) are appended to the computations output list."""
        if isinstance(obj, list):
            for o in obj:
                serialized_bytes = serialize_for_hashing(o)
                cid = get_cid_for_bytes(serialized_bytes, self._store)
                self._output_cids.append(cid)
        else:
            serialized_bytes = serialize_for_hashing(obj)
            cid = get_cid_for_bytes(serialized_bytes, self._store)
            self._output_cids.append(cid)

        return self

    def set_computation_cid(self, cid: CID) -> "Computation":
        """Sets the computation CID with the provided cid."""
        if isinstance(cid, CID):
            self._computation_cid = cid
        return self

    def set_computation_path(self, path: Union[Path, str]) -> "Computation":
        """Resolves the path; cids the contents, then sets the computations CID."""
        self._computation_cid = __cid_path__(path, self._store)
        return self

    def set_computation_object(self, obj: Any) -> "Computation":
        """Serializes the obj, then calculates the CID for the serialized bytes and sets the computations cid."""
        serialized_bytes = serialize_for_hashing(obj)
        cid = get_cid_for_bytes(serialized_bytes, self._store)
        self._computation_cid = cid
        return self

    def set_start_time(self, time: Union[datetime, str]) -> "Computation":
        """Sets when the computation started, as a timezone-aware datetime or an RFC 3339 string.

        This is when the work began, not when the statement is registered.
        """
        self._start_time = __rfc3339__(time)
        return self

    def set_end_time(self, time: Union[datetime, str]) -> "Computation":
        """Sets when the computation ended, as a timezone-aware datetime or an RFC 3339 string."""
        self._end_time = __rfc3339__(time)
        return self

    def set_span(
        self, trace_id: str, span_id: str, parent_span_id: Optional[str] = None
    ) -> "Computation":
        """Records which OpenTelemetry span this computation was, and the span it ran inside.

        Takes the W3C trace-context hex IDs (32 and 16 digits) and stores each span as a
        `urn:uuid` derived from the trace and span ID (see `otel_span_urn`). A child names
        its parent, so it can be finalized as soon as it ends, before its parent has.
        """
        self._span = otel_span_urn(trace_id, span_id)
        self._parent_span = otel_span_urn(trace_id, parent_span_id) if parent_span_id else None
        return self

    @property
    def statement_id(self) -> Optional[CID]:
        """The ComputationRegistration statement ID, once finalized."""
        return self._statement_id

    def finalize(self) -> "Computation":
        """Creates the computation statement, and adds a metadata statement for the computation statement."""
        statement_ids = add_computation_statement(
            inputs=self._input_cids,
            outputs=self._output_cids,
            computation=self._computation_cid,
            start_time=self._start_time,
            end_time=self._end_time,
            span=self._span,
            parent_span=self._parent_span,
            _skip_proof=self._skip_proof,
            context=self._ctx,
        )
        self._statement_id = statement_ids[0]

        self._metadata.create_statement(statement_ids[0], self._skip_proof, context=self._ctx)

        return self
