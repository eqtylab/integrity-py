import json
import logging
import os
from enum import Enum
from pathlib import Path
from typing import Any, Optional, Union, cast

import dill as pickle

from eqty_sdk import config
from eqty_sdk._rust import (
    Graph as Context,
    statements as eqty_core_statements,
)
from eqty_sdk.core import (
    get_cid_for_bytes,
    get_cid_for_path,
)
from eqty_sdk.metadata import Metadata
from eqty_sdk.statements import (
    add_data_statement,
    add_governance_statement,
    add_metadata_statement,
)
from eqty_sdk.types.cid import Cid
from eqty_sdk.types.declaration import Declaration

logger = logging.getLogger("eqty.sdk.Asset")


# This determines the icon that is used on the graph.
# These are the 'known' types, but actual value can be anything
class AssetType(Enum):
    ATTRIBUTION = "Attribution"
    BENCHMARK = "Benchmark"
    BENCHMARK_RESULT = "Benchmark_Result"
    CERTIFICATE = "Certificate"
    CODE = "Code"
    CUSTOM = "Custom"
    DATABASE = "Database"
    DATASET = "Dataset"
    DOCUMENT = "Document"
    MEDIA = "Media"
    MODEL = "Model"
    TOKEN = "Token"


def _init_path_input(path: Union[str, Path]) -> Path:
    """Gets the resolved Path."""
    path2 = Path(path) if isinstance(path, str) else path
    resolved_path = path2.resolve()
    return resolved_path


def serialize_for_hashing(obj: Any) -> bytes:
    """Serializes the object to a format suitable for hashing based on its type."""
    if isinstance(obj, str):
        return obj.encode("utf-8")  # Encode strings
    elif isinstance(obj, (int, float)):
        return str(obj).encode("utf-8")  # Encode numbers as strings
    elif isinstance(obj, list) or isinstance(obj, dict):
        return json.dumps(obj).encode("utf-8")  # JSON-encode complex data structures
    elif hasattr(obj, "serialize_for_hashing"):  # Check for custom implementation
        return cast(bytes, obj.serialize_for_hashing())
    # Because 'object' types can be arbitrary, we'll pickle them. This is not recommended for untrusted data.
    elif isinstance(obj, object):
        try:
            # Check if the object has a 'model' attribute
            if hasattr(obj, "model"):
                # Serialize the state dictionary of the model
                state_dict = getattr(obj, "model").state_dict()
                return cast(bytes, pickle.dumps(state_dict))
            return cast(bytes, pickle.dumps(obj))  # Pickle objects
        except (pickle.PickleError, TypeError) as e:
            raise TypeError(f"Unsupported data type for hashing: {type(obj)} - {e}")
    # Add more cases for other data types as needed
    else:
        raise TypeError(f"Unsupported data type for hashing: {type(obj)}")


def get_asset_name(asset_type: Union[AssetType, str], cid: str) -> str:
    if isinstance(asset_type, AssetType):
        return f"{asset_type.value}-{cid[-4:]}"
    else:
        return f"{asset_type}-{cid[-4:]}"


def _should_skip_proof(**kwargs):
    """Checks if skip_proof is set in kwargs, then checks the env var, finally defaults to False."""
    skip = kwargs.pop("skip_proof", None)
    if skip is not None:
        return skip
    else:
        return os.getenv("EQTY_SKIP_PROOF", "").lower() == "true"


class TypedAsset:
    """Base class for typed assets. Subclasses just need to set _asset_type class variable."""

    _asset_type: AssetType

    @classmethod
    def from_path(cls, path: Union[str, Path], store: Optional[bool] = None, **kwargs) -> "Asset":
        return Asset._from_path(path, cls._asset_type, store=store, **kwargs)

    @classmethod
    def from_cid(cls, cid: Union[Cid, str], **kwargs) -> "Asset":
        cid_str = cid.cid if isinstance(cid, Cid) else cid
        return Asset._from_cid(cid_str, cls._asset_type, **kwargs)

    @classmethod
    def from_object(cls, obj: Any, store: Optional[bool] = None, **kwargs) -> "Asset":
        return Asset._from_object(obj, cls._asset_type, store=store, **kwargs)

    @classmethod
    def with_context(cls, ctx: Context) -> Any:
        return Asset._factory_with_context(ctx, cls._asset_type)


class Asset:
    def __init__(
        self,
        obj: Any,
        asset_type: Union[AssetType, str],
        cid: str,
        is_dir: bool,
        custom_ctx: Optional[Context],
        **kwargs,
    ):
        self._ctx = custom_ctx

        self._value: Any = obj
        self._skip_proof = _should_skip_proof(**kwargs)

        self._cid = cid
        self._is_dir = is_dir
        self.statement_ids: list[str] = []

        if isinstance(asset_type, AssetType):
            kwargs.update({"assetType": asset_type.value})
        else:
            kwargs.update({"assetType": asset_type})

        self._metadata = Metadata(**kwargs)

        if not kwargs.get("skip_registration", False):
            try:
                self._create_eqty_statements()

            except RuntimeError as e:
                logger.error(f"Error creating new asset: {e}")
                raise e

    @staticmethod
    def _from_object(
        obj: Any,
        asset_type: Union[AssetType, str],
        ctx: Optional[Context] = None,
        store: Optional[bool] = None,
        **kwargs,
    ) -> "Asset":
        serialized_bytes = serialize_for_hashing(obj)
        cid = get_cid_for_bytes(serialized_bytes, store)
        kwargs.setdefault("name", get_asset_name(asset_type, cid))

        asset = Asset(obj, asset_type, cid, is_dir=False, custom_ctx=ctx, **kwargs)
        return asset

    @staticmethod
    def _from_path(
        path: Union[Path, str],
        asset_type: Union[AssetType, str],
        ctx: Optional[Context] = None,
        store: Optional[bool] = None,
        **kwargs,
    ) -> "Asset":
        resolved_path = _init_path_input(path)
        cid = get_cid_for_path(resolved_path, store)
        is_dir = resolved_path.is_dir()
        kwargs.setdefault("name", get_asset_name(asset_type, cid))

        asset = Asset(resolved_path, asset_type, cid, is_dir, custom_ctx=ctx, **kwargs)
        return asset

    @staticmethod
    def _from_cid(
        cid: str, asset_type: Union[AssetType, str], ctx: Optional[Context], **kwargs
    ) -> "Asset":
        kwargs.setdefault("name", get_asset_name(asset_type, cid))

        asset = Asset(cid, asset_type, cid, is_dir=False, custom_ctx=ctx, **kwargs)
        return asset

    @staticmethod
    def _factory_with_context(ctx: Context, asset_type: AssetType):
        class _Factory:
            def from_path(
                self, path: Union[str, Path], store: Optional[bool] = None, **kwargs
            ) -> "Asset":
                return Asset._from_path(path, asset_type, ctx, store, **kwargs)

            def from_cid(self, cid: Union[Cid, str], **kwargs) -> "Asset":
                cid_str = cid.cid if isinstance(cid, Cid) else cid
                return Asset._from_cid(cid_str, asset_type, ctx, **kwargs)

            def from_object(self, obj: Any, store: Optional[bool] = None, **kwargs) -> "Asset":
                return Asset._from_object(obj, asset_type, ctx, store, **kwargs)

        return _Factory()

    @property
    def name(self) -> str:
        """Get the name of the asset."""
        return cast(str, self._metadata.name)

    @property
    def cid(self) -> str:
        """Get the CID of the asset."""
        return self._cid

    @property
    def asset_type(self) -> str:
        """Get the type of the asset."""
        return self._metadata._additional_metadata.get("assetType") or ""

    @property
    def value(self) -> Any:
        """Get the underlying value."""
        return self._value

    def add_declaration(self, declaration: Declaration) -> "Asset":
        document_cid = declaration.cid()
        ids = add_governance_statement(self.cid, document_cid, self._skip_proof)
        self.statement_ids.extend(ids)
        return self

    def _create_eqty_statements(self) -> None:
        """Creates DataStatement, MetadataStatement, and VcStatement."""
        self.statement_ids.extend(add_data_statement([self.cid], self._skip_proof))
        self.statement_ids.extend(add_metadata_statement(self.cid, self._metadata.to_json_str(), self._skip_proof))

        if config.get_generate_model_signing_signatures() and self._is_dir:
            _, _, include_symlinks = config.get_cid_ignore_rules()
            eqty_core_statements.create_model_signing_statement(
                collection_cid=self.cid,
                blobs_dir=config.blob_dir(),
                model_signing_name=self._metadata.name or "Unnamed Asset",
                allow_symlinks=include_symlinks,
                ignore_paths=[],  # TODO: populate this, ok for now, just makes recreation require more out-of-band info
                timestamp=None,
            )

    def __repr__(self) -> str:
        return f"Asset({self._value!r})"

    def __str__(self) -> str:
        return cast(str, self._value.__str__())

    def __getattr__(self, key):
        logger.info(f"Getting key: {key}")
        # Handle specific attributes directly
        if key in {"asset_type", "cid", "name", "value", "add_attribute"}:
            return object.__getattribute__(self, key)

        # Attempt to get the attribute from the wrapped object
        try:
            return getattr(self._value, key)
        except AttributeError:
            # Check the _metadata dictionary if the key is not found
            if key in self._metadata:
                return self._metadata[key]
            else:
                raise AttributeError(
                    f"'{type(self._value).__name__}' object has no attribute '{key}'"
                )

    def __setattr__(self, key, value):
        # logger.info(f"Setting key: {key}")
        # Special handling to set attributes on the wrapper, not the wrapped object
        if key in {
            "_ctx",
            "_value",
            "_name",
            "_cid",
            "_is_dir",
            "_metadata",
            "_asset_type",
            "_skip_proof",
            "statement_ids",
        }:
            object.__setattr__(self, key, value)
        else:
            setattr(self._value, key, value)

    def __getitem__(self, key):
        # Delegate indexing to the wrapped object
        return self._value[key]

    def __setitem__(self, key, value):
        # Delegate item assignment to the wrapped object
        self._value[key] = value

    def __hash__(self):
        return hash(self._value)

    def __iter__(self):
        try:
            # Try to delegate iteration to the wrapped object
            return iter(self._value)
        except TypeError:
            raise TypeError(f"{type(self._value).__name__} object is not iterable")

    def __len__(self):
        try:
            # Try to get the length of the wrapped object
            return len(self._value)
        except TypeError:
            raise TypeError(f"{type(self._value).__name__} object has no len()")

    def __eq__(self, other):
        """Check if two Asset objects are equal based on their underlying values."""
        if not isinstance(other, Asset):
            if self._value is None and other is None:
                return True
            if self._value is None or other is None:
                return False
            return self._value == other
        if self._value is None and other._value is None:
            return True
        if self._value is None or other._value is None:
            return False
        return self._value == other._value

    def __add__(self, other):
        """Overload the + operator to support adding Asset objects if their underlying types are compatible,
        otherwise fallback to delegating to the underlying value.
        """
        try:
            if (
                isinstance(self._value, list)
                and isinstance(other, Asset)
                and isinstance(other._value, list)
            ):
                # If both underlying values are lists, add them directly
                return self._value + other._value
            elif isinstance(other, Asset):
                return self._value + other._value
            else:
                # Delegate addition to the underlying value if possible
                return self._value + other
        except TypeError:
            # Raise specific error if addition fails entirely
            raise TypeError("Cannot add Asset objects together")

    def __mul__(self, other):
        """Overload the * operator to potentially support multiplication of Asset objects
        based on the underlying value's behavior.
        """
        try:
            if isinstance(other, (int, float)) and isinstance(self._value, (int, float)):
                # If both are numbers, multiply them directly
                return self._value * other
            elif isinstance(other, Asset):
                return self._value * other._value
            else:
                # Delegate multiplication to the underlying value if possible
                return self._value * other
        except TypeError:
            # Raise specific error if multiplication fails entirely
            raise TypeError(f"Unsupported operand types for *: 'Asset' and '{type(other)}'")

    def __truediv__(self, other):
        """Overload the / operator to potentially support division of Asset objects
        based on the underlying value's behavior.
        """
        try:
            if isinstance(other, (int, float)) and isinstance(self._value, (int, float)):
                # If both are numbers, divide them directly
                return self._value / other
            elif isinstance(other, Asset):
                return self._value / other._value
            else:
                # Delegate division to the underlying value if possible
                return self._value / other
        except TypeError:
            # Raise specific error if division fails entirely
            raise TypeError(f"Unsupported operand types for /: 'Asset' and '{type(other)}'")

    def __floordiv__(self, other):
        """Overload the // operator to potentially support floor division of Asset objects
        based on the underlying value's behavior.
        """
        try:
            if isinstance(other, (int, float)) and isinstance(self._value, (int, float)):
                # If both are numbers, divide them directly
                return self._value // other
            elif isinstance(other, Asset):
                return self._value // other._value
            else:
                # Delegate floor division to the underlying value if possible
                return self._value // other
        except TypeError:
            # Raise specific error if floor division fails entirely
            raise TypeError(f"Unsupported operand types for //: 'Asset' and '{type(other)}'")

    def __mod__(self, other):
        """Overload the % operator to potentially support modulo of Asset objects
        based on the underlying value's behavior.
        """
        try:
            if isinstance(other, (int, float)) and isinstance(self._value, (int, float)):
                # If both are numbers, divide them directly
                return self._value % other
            elif isinstance(other, Asset):
                return self._value % other._value
            else:
                # Delegate modulo to the underlying value if possible
                return self._value % other
        except TypeError:
            # Raise specific error if modulo fails entirely
            raise TypeError(f"Unsupported operand types for %: 'Asset' and '{type(other)}'")

    def __pow__(self, other):
        """Overload the ** operator to potentially support exponentiation of Asset objects
        based on the underlying value's behavior.
        """
        try:
            if isinstance(other, (int, float)) and isinstance(self._value, (int, float)):
                # If both are numbers, divide them directly
                return self._value**other
            elif isinstance(other, Asset):
                return self._value**other._value
            else:
                # Delegate exponentiation to the underlying value if possible
                return self._value**other
        except TypeError:
            # Raise specific error if exponentiation fails entirely
            raise TypeError(f"Unsupported operand types for **: 'Asset' and '{type(other)}'")

    def __sub__(self, other):
        """Overload the - operator to potentially support subtraction of Asset objects
        based on the underlying value's behavior.
        """
        try:
            if isinstance(other, (int, float)) and isinstance(self._value, (int, float)):
                # If both are numbers, subtract them directly
                return self._value - other
            elif isinstance(other, Asset):
                return self._value - other._value
            else:
                # Delegate subtraction to the underlying value if possible
                return self._value - other
        except TypeError:
            # Raise specific error if subtraction fails entirely
            raise TypeError(f"Unsupported operand types for -: 'Asset' and '{type(other)}'")
