import logging
import os
from pathlib import Path
from typing import Any, List, Optional, Union, cast

from eqty_sdk._rust import (
    CID,
    Graph as Context,
    get_cid_for_bytes,
    get_cid_for_path,
)
from eqty_sdk.asset import serialize_for_hashing
from eqty_sdk.errors import UsageError
from eqty_sdk.metadata import Metadata
from eqty_sdk.statements import add_computation_statement

logger = logging.getLogger("eqty.sdk.computation")


def __cid_path__(path: Union[str, Path], store: Optional[bool] = None) -> str:
    """Resolves the path, and then calculates the CID."""
    path2 = Path(path) if isinstance(path, str) else path
    resolved_path = path2.resolve()

    if resolved_path.is_dir():
        return get_cid_for_path(resolved_path, store)
    else:
        try:
            return get_cid_for_path(resolved_path, store)
        except (IOError, OSError):
            raise UsageError(f"The input path '{resolved_path}' was not found.")


class Computation:
    def __init__(self):
        raise TypeError("Use Computation.new() to create instances of this class.")

    def __init_internal__(
        self, ctx: Optional[Context], metadata: Metadata, skip_proof: Optional[bool] = None
    ):
        self._ctx = ctx
        self._metadata = metadata
        self._input_cids: List[str] = []
        self._output_cids: List[str] = []
        self._computation_cid: Union[str, None] = None

        if skip_proof is not None:
            self._skip_proof = skip_proof
        else:
            self._skip_proof = os.getenv("EQTY_SKIP_PROOF", "").lower() == "true"

    @classmethod
    def new(cls, **kwargs) -> "Computation":
        skip_proof = kwargs.pop("skip_proof", None)

        metadata = Metadata(**kwargs)
        instance = object.__new__(cls)
        instance.__init_internal__(None, metadata, skip_proof)
        return instance

    @staticmethod
    def with_context(ctx: Context):
        class _Factory:
            def new(self, **kwargs) -> "Computation":
                skip_proof = kwargs.pop("skip_proof", None)

                metadata = Metadata(**kwargs)
                instance = object.__new__(Computation)
                instance.__init_internal__(ctx, metadata, skip_proof)
                return instance

        return _Factory()

    def add_input_cid(self, cid: Union[List[CID], List[str], CID, str]) -> "Computation":
        """Adds the CID(s) to the computations input list."""
        if isinstance(cid, CID):
            self._input_cids.append(cid.cid)
        elif isinstance(cid, list) and all(isinstance(item, CID) for item in cid):
            cids = [cast(CID, item).cid for item in cid]
            self._input_cids.extend(cids)
        elif isinstance(cid, str):
            self._input_cids.append(cid)
        elif isinstance(cid, list) and all(isinstance(item, str) for item in cid):
            self._input_cids.extend(cast(List[str], cid))
        else:
            raise ValueError("Invalid type for cid")

        return self

    def add_input_path(self, path: Union[List[Path], List[str], Path, str]) -> "Computation":
        """Resolves the provide path(s) and adds the computed CID(s) to the computations input list."""
        if isinstance(path, Path):
            self._input_cids.append(__cid_path__(path))
        elif isinstance(path, list) and all(isinstance(item, Path) for item in path):
            self._input_cids.extend([__cid_path__(p) for p in path])
        elif isinstance(path, str):
            self._input_cids.append(__cid_path__(path))
        elif isinstance(path, list) and all(isinstance(item, str) for item in path):
            for p in path:
                self._input_cids.append(__cid_path__(p))
        else:
            raise ValueError("Invalid type for path")
        return self

    def add_input_object(self, obj: Union[List[Any], Any]) -> "Computation":
        """Serializes the obj(s), then calculates the CID for the serialized bytes. The CID(s) are appended to the computations input list."""
        if isinstance(obj, list):
            for o in obj:
                serialized_bytes = serialize_for_hashing(o)
                cid = get_cid_for_bytes(serialized_bytes)
                self._input_cids.append(cid)
        else:
            serialized_bytes = serialize_for_hashing(obj)
            cid = get_cid_for_bytes(serialized_bytes)
            self._input_cids.append(cid)

        return self

    def add_output_cid(self, cid: Union[List[CID], List[str], CID, str]) -> "Computation":
        """Adds the CID(s) to the computations output list."""
        if isinstance(cid, CID):
            self._output_cids.append(cid.cid)
        elif isinstance(cid, list) and all(isinstance(item, CID) for item in cid):
            cids = [cast(CID, item).cid for item in cid]
            self._output_cids.extend(cids)
        elif isinstance(cid, str):
            self._output_cids.append(cid)
        elif isinstance(cid, list) and all(isinstance(item, str) for item in cid):
            self._output_cids.extend(cast(List[str], cid))
        else:
            raise ValueError("Invalid type for cid")

        return self

    def add_output_path(self, path: Union[List[Path], List[str], Path, str]) -> "Computation":
        """Resolves the provide path(s) and adds the computed CID(s) to the computations output list."""
        if isinstance(path, Path):
            self._output_cids.append(__cid_path__(path))
        elif isinstance(path, list) and all(isinstance(item, Path) for item in path):
            self._output_cids.extend([__cid_path__(p) for p in path])
        elif isinstance(path, str):
            self._output_cids.append(__cid_path__(path))
        elif isinstance(path, list) and all(isinstance(item, str) for item in path):
            for p in path:
                self._output_cids.append(__cid_path__(p))
        else:
            raise ValueError("Invalid type for path")
        return self

    def add_output_object(self, obj: Union[List[Any], Any]) -> "Computation":
        """Serializes the obj(s), then calculates the CID for the serialized bytes. The CID(s) are appended to the computations output list."""
        if isinstance(obj, list):
            for o in obj:
                serialized_bytes = serialize_for_hashing(o)
                cid = get_cid_for_bytes(serialized_bytes)
                self._output_cids.append(cid)
        else:
            serialized_bytes = serialize_for_hashing(obj)
            cid = get_cid_for_bytes(serialized_bytes)
            self._output_cids.append(cid)

        return self

    def set_computation_cid(self, cid: Union[CID, str]) -> "Computation":
        """Sets the computation CID with the provided cid."""
        if isinstance(cid, CID):
            self._computation_cid = cid.cid
        elif isinstance(cid, str):
            self._computation_cid = cid
        return self

    def set_computation_path(self, path: Union[Path, str]) -> "Computation":
        """Resolves the path; cids the contents, then sets the computations CID."""
        self._computation_cid = __cid_path__(path)
        return self

    def set_computation_object(self, obj: Any) -> "Computation":
        """Serializes the obj, then calculates the CID for the serialized bytes and sets the computations cid."""
        serialized_bytes = serialize_for_hashing(obj)
        cid = get_cid_for_bytes(serialized_bytes)
        self._computation_cid = cid
        return self

    def finalize(self) -> "Computation":
        """Creates the computation statement, and adds a metadata statement for the computation statement."""
        statement_ids = add_computation_statement(
            inputs=self._input_cids,
            outputs=self._output_cids,
            computation=self._computation_cid,
            skip_proof=self._skip_proof,
            graph=self._ctx,
        )

        self._metadata.create_statement(statement_ids[0], self._skip_proof)

        return self
