import inspect
import json
import logging
import os
from typing import Any, Callable, Dict, List, Optional, cast

from eqty_sdk._rust import (
    statements as eqty_core_statements,
    stream as eqty_core_stream,
)
from eqty_sdk.asset import Asset, AssetType, Code, Custom, Dataset, Model
from eqty_sdk.context import Context, ContextType, OriginalCtx
from eqty_sdk.core import get_cid_for_bytes
from eqty_sdk.errors import UsageError
from eqty_sdk.feature_flags import FEATURE_FLAGS, FeatureFlags, feature_gate_when_disabled
from eqty_sdk.metadata import Metadata
from eqty_sdk.statements import add_computation_statement
from eqty_sdk.statements.common import add_vc_statement

logger = logging.getLogger("eqty.sdk.computation")


class Compute:
    """A wrapper class to hold and execute a function.

    This class is used to wrap a function and execute it with the provided arguments.

    Args:
        func: The function to be wrapped.
        metadata: Additional metadata associated with the computation.

    Returns:
        A Compute object that wraps the provided function.

    """

    @property
    def name(self) -> str:
        value = self.metadata.name
        return cast(str, value)

    @property
    def cid(self) -> str:
        value = self._cid
        return cast(str, value)

    @property
    def skip_proof(self) -> bool:
        value = self._skip_proof
        return cast(bool, value)

    @property
    def source_code(self) -> str:
        return self._source_code

    def __init__(
        self,
        func: Callable[..., Any],
        metadata: Optional[Dict[str, Any]] = None,
        store: Optional[None] = None,
        ctx: Optional[ContextType] = None,
        **kwargs,
    ) -> None:
        logger.debug("Initalizing Compute")

        if FeatureFlags.is_disabled(FEATURE_FLAGS.GRAPH_IDS):
            self._ctx = ctx if ctx is not None else Context()
        else:
            # get current root context
            pass

        if metadata is None:
            metadata = {}

        self.metadata = Metadata(**metadata)

        skip = kwargs.pop("skip_proof", None)
        if skip is not None:
            self._skip_proof = skip
        else:
            self._skip_proof = os.getenv("EQTY_SKIP_PROOF", "").lower() == "true"

        # pointer to the wrapped fn so we can call it later
        self._func = func

        self._source_code = inspect.getsource(self._func)
        func_bytes = self._source_code.encode("utf-8")
        self._cid = get_cid_for_bytes(func_bytes)

        # holds list of statement ids (inputs, outputs, metadata, vc, etc) that are created during the computation so
        # that attributes can be applied to all statements associated with this computation
        self.statement_ids = [self._cid]

        # local flag to track if any hashed data should be stored as a blob
        self._store = store

        if FeatureFlags.is_disabled(FEATURE_FLAGS.GRAPH_IDS):
            # attributes to apply to statements. Needs tracked due to async fns not having all statements available when
            # the compute returns to the caller
            self._attributes: dict[str, str] = {}

        # source code is created as an input asset to the compute node
        self._code_asset = Code.from_object(
            self._source_code,
            store,
            skip_proof=self.skip_proof,
            name=self._func.__name__,
            **(
                {"description": self._func.__doc__} if self._func.__doc__ is not None else {}
            ),  # only include 'description' if the fn doc string exists
        )

    def __create_asset__(self, output_type: Optional[str], item: Any) -> Any:
        """Creates an Eqty Asset from `item` of type=output_type."""
        if item is None:
            return None

        if isinstance(item, Asset):
            return item

        if isinstance(item, tuple) and all(isinstance(i, Asset) for i in item):
            return item

        if isinstance(item, tuple):
            return tuple(self.__create_asset__(output_type, i) for i in item)

        if isinstance(item, list) and len(item) > 0:
            if all(isinstance(i, Asset) for i in item):
                return item
            else:
                return [
                    self.__create_asset__(output_type, list_item)
                    for list_item in item
                    if list_item is not None
                ]

        if output_type == "dataset":
            return Dataset.from_object(
                item,
                self._store,
                skip_proof=self.skip_proof,
            )
        elif output_type == "model":
            return Model.from_object(
                item,
                self._store,
                skip_proof=self.skip_proof,
            )
        else:
            return Custom.from_object(
                item,
                AssetType.CUSTOM,
                self._store,
                name="Custom",
                skip_proof=self.skip_proof,
            )

    def __args_to_assets__(self, args) -> List[Asset]:
        """Convert the input arguments to Assets and returns the list of assets."""
        inputs: List[Any] = []
        # add the souce code to the list of input assets
        inputs.append(self._code_asset)

        # Get the parameter names of the wrapped function to be used on the graph
        sig = inspect.signature(self._func)
        param_names = list(sig.parameters.keys())

        for param_name, arg in zip(param_names, args):
            if isinstance(arg, Asset):
                logger.debug(f"Adding asset to inputs array. {arg.cid}")
                self.statement_ids.extend(arg.statement_ids)
                inputs.append(arg)
            elif isinstance(arg, list) and all(isinstance(item, Asset) for item in arg):
                logger.debug(f"Adding list of assets to inputs array. {[item.cid for item in arg]}")
                inputs.extend(arg)
            elif hasattr(arg, "to_eqty_asset"):
                # special fn to allow users to create a custom fn defining how to convert from their type to an asset
                inputs.append(arg.to_eqty_asset())
            elif arg is not None:
                asset = Custom.from_object(
                    arg,
                    AssetType.CUSTOM,
                    self._store,
                    name=param_name,
                    skip_proof=self.skip_proof,
                )
                inputs.append(asset)
                self.statement_ids.extend(asset.statement_ids)

        return inputs

    def __results_to_assets__(self, result: Any) -> Any:
        """Converts a Tuple, List, or single value to an equivalent Asset type."""
        if result is None:
            raise UsageError("The captured function must return a result")

        output_type = self.metadata.output_type

        result = self.__create_asset__(output_type, result)
        return result

    async def __call_async_gen__(self, input_cids, *args: Any, **kwargs: Any) -> Any:
        stream = self._func(*args, **kwargs)

        stream_uuid = await eqty_core_stream.create(input_cids)

        buffer = []

        async for chunk in stream:
            if isinstance(chunk, str):
                chunk_bytes = chunk.encode("utf-8")
            elif isinstance(chunk, bytes):
                chunk_bytes = chunk
            elif isinstance(chunk, (int, float, complex)):  # Check for numbers
                chunk_bytes = str(chunk).encode("utf-8")
            elif isinstance(chunk, dict) or hasattr(chunk, "__dict__"):  # Check for objects
                try:
                    chunk_bytes = json.dumps(chunk, default=lambda o: o.__dict__).encode("utf-8")
                except TypeError as e:
                    raise ValueError(f"Unable to encode chunk of type {type(chunk)}: {e}")
            else:
                msg = f"Unsupported type for chunk: {type(chunk)}"
                logger.error(msg)
                raise TypeError(msg)

            await eqty_core_stream.update(stream_uuid, chunk_bytes)
            buffer.append(chunk)
            yield chunk

        # Finalizing the stream will add the computation statement
        result = await eqty_core_stream.finalize(stream_uuid)
        compute_cid = result.get("compute_id")
        stream = result.get("stream")
        logger.debug(f"Stream committed '{stream_uuid}'. Computation CID:'{compute_cid}'")
        self.statement_ids.append(compute_cid)
        vc_id = add_vc_statement(compute_cid, None, self.skip_proof)
        if vc_id:
            self.statement_ids.append(vc_id)

        ids = self.metadata.create_statement(compute_cid, self.skip_proof)
        self.statement_ids.extend(ids)

        stream_cid = get_cid_for_bytes(stream, self._store)
        asset = Custom.from_cid(
            stream_cid,
            AssetType.CUSTOM,
            name=f"{self.name}-stream",
            skip_proof=self.skip_proof,
        )
        asset.add_attribute(**self._attributes)
        self.__add_attribute__(**self._attributes)

        if isinstance(self._ctx, OriginalCtx):
            if self._ctx.project_id:
                asset.add_attribute(__project_id=self._ctx.project_id)
                self.__add_attribute__(__project_id=self._ctx.project_id)

    async def __call_async__(self, input_cids, *args: Any, **kwargs: Any) -> Any:
        """Execute the wrapped function asynchronously."""
        result = await self._func(*args, **kwargs)
        self.__finalize__(input_cids, result)
        logger.debug("Finalizing async fn")
        self.__add_attribute__(**self._attributes)
        return result

    def __call_sync__(self, input_cids, *args: Any, **kwargs: Any) -> Any:
        """Execute the wrapped function synchronously."""
        result = self._func(*args, **kwargs)
        self.__finalize__(input_cids, result)
        return result

    def __call__(self, *args: Any, **kwargs: Any) -> Any:
        """Execute the wrapped function with the provided arguments.

        Returns:
            The result of executing the wrapped function.

        """
        logger.debug("Prepping func inputs")

        inputs = self.__args_to_assets__(args)
        for i in inputs:
            self.statement_ids.extend(i.statement_ids)
        input_cids = [i.cid for i in inputs]

        if inspect.iscoroutinefunction(self._func):
            logger.debug(f"'{self.name}' is async")
            return self.__call_async__(input_cids, *args, **kwargs)

        elif inspect.isasyncgenfunction(self._func):
            logger.debug(f"'{self.name}' is async generator")
            return self.__call_async_gen__(input_cids, *args, **kwargs)

        else:
            logger.debug(f"'{self.name}' is standard function")
            return self.__call_sync__(input_cids, *args, **kwargs)

    def __finalize__(self, input_cids, result: Any) -> List[str]:
        """Finalize the computation by adding the computation statement and metadata statements."""

        def extract_cids(asset) -> List[str]:
            """Recursively extract cids from asset, which can be a tuple, list, or an Asset."""
            cids = []
            if isinstance(asset, tuple) or isinstance(asset, list):
                for item in asset:
                    cids.extend(extract_cids(item))
            elif asset is not None:
                self.statement_ids.extend(asset.statement_ids)
                cids.append(asset.cid)
            return cids

        result_asset = self.__results_to_assets__(result)
        output_cids = extract_cids(result_asset)

        statement_ids = add_computation_statement(
            inputs=input_cids,
            outputs=output_cids,
            computation=None,
            skip_proof=self.skip_proof,
        )
        self.statement_ids.extend(statement_ids)

        ids = self.metadata.create_statement(statement_ids[0], self.skip_proof)
        self.statement_ids.extend(ids)

        if isinstance(self._ctx, OriginalCtx):
            if self._ctx.project_id:
                self.__add_attribute__(__project_id=self._ctx.project_id)

        return output_cids

    @feature_gate_when_disabled(FEATURE_FLAGS.GRAPH_IDS)
    def __add_attribute__(self, **kwargs) -> None:
        logger.info(f"adding attributes {kwargs} to {self.statement_ids}")
        eqty_core_statements.add_attributes_to_statements(self.statement_ids, kwargs)
        self._attributes = kwargs
