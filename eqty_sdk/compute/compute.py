import inspect
import json
import logging
from typing import Any, Callable, Dict, List, Optional, cast

from eqty_sdk._rust import (
    CID,
    Context,
    get_cid_for_bytes,
    statements,
    stream as eqty_core_stream,
)
from eqty_sdk.asset import Asset, AssetType, Code, Custom, Dataset, Model
from eqty_sdk.errors import UsageError
from eqty_sdk.metadata import Metadata
from eqty_sdk.statements import add_computation_statement

logger = logging.getLogger("eqty.sdk.computation")


class Compute:
    """A wrapper class to hold and execute a function.

    This class is used to wrap a function and execute it with the provided arguments.

    Args:
        func: The function to be wrapped.
        metadata: Additional metadata associated with the computation.

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
    def source_code(self) -> str:
        return self._source_code

    def __init__(
        self,
        func: Callable[..., Any],
        metadata: Optional[Dict[str, Any]] = None,
        _store: Optional[None] = None,
        ctx: Optional[Context] = None,
        **kwargs,
    ) -> None:
        logger.debug("Initalizing Compute")

        self._ctx = ctx

        if metadata is None:
            metadata = {}

        self.metadata = Metadata(**metadata)

        self._skip_proof = kwargs.pop("_skip_proof", None)

        # pointer to the wrapped fn so we can call it later
        self._func = func

        self._source_code = inspect.getsource(self._func)
        func_bytes = self._source_code.encode("utf-8")
        self._cid = get_cid_for_bytes(func_bytes)

        # local flag to track if any hashed data should be stored as a blob
        self._store = _store

        # source code is created as an input asset to the compute node
        self._code_asset = Code.from_object(
            self._source_code,
            _store,
            _skip_proof=self._skip_proof,
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
                _skip_proof=self._skip_proof,
            )
        elif output_type == "model":
            return Model.from_object(
                item,
                self._store,
                _skip_proof=self._skip_proof,
            )
        else:
            return Custom.from_object(
                item,
                AssetType.CUSTOM,
                self._store,
                name="Custom",
                _skip_proof=self._skip_proof,
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
                inputs.append(arg)
            elif isinstance(arg, list) and all(isinstance(item, Asset) for item in arg):
                logger.debug(f"Adding list of assets to inputs array. {[item.cid for item in arg]}")
                inputs.extend(arg)
            elif hasattr(arg, "to_eqty_asset"):
                # special fn to allow users to define a custom conversion fn from their type to an asset
                inputs.append(cast(Any, arg).to_eqty_asset())
            elif arg is not None:
                asset = Custom.from_object(
                    arg,
                    AssetType.CUSTOM,
                    self._store,
                    name=param_name,
                    _skip_proof=self._skip_proof,
                )
                inputs.append(asset)

        return inputs

    def __results_to_assets__(self, result: Any) -> Any:
        """Converts a Tuple, List, or single value to an equivalent Asset type."""
        if result is None:
            raise UsageError("The captured function must return a result")

        output_type = self.metadata.output_type

        result = self.__create_asset__(output_type, result)
        return result

    async def __call_async_gen__(self, input_cids: List[CID], *args: Any, **kwargs: Any) -> Any:
        stream = self._func(*args, **kwargs)

        stream_uuid = await eqty_core_stream.create(input_cids, None, None, None)

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
        result = await eqty_core_stream.finalize(stream_uuid, [], context=self._ctx)
        compute_cid = result.get("compute_id")
        stream = result.get("stream")
        logger.debug(f"Stream committed '{stream_uuid}'. Computation CID:'{compute_cid}'")
        statements.add_vc_statement(compute_cid)

        self.metadata.create_statement(compute_cid, self._skip_proof, context=self._ctx)

        stream_cid = get_cid_for_bytes(stream, self._store)
        Custom.from_cid(
            stream_cid,
            AssetType.CUSTOM,
            name=f"{self.name}-stream",
            _skip_proof=self._skip_proof,
        )

    async def __call_async__(self, input_cids, *args: Any, **kwargs: Any) -> Any:
        """Execute the wrapped function asynchronously."""
        result = await self._func(*args, **kwargs)
        self.__finalize__(input_cids, result)
        logger.debug("Finalizing async fn")
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

    def __finalize__(self, input_cids, result: Any) -> List[CID]:
        """Finalize the computation by adding the computation statement and metadata statements."""

        def extract_cids(asset) -> List[CID]:
            """Recursively extract cids from asset, which can be a tuple, list, or an Asset."""
            cids = []
            if isinstance(asset, tuple) or isinstance(asset, list):
                for item in asset:
                    cids.extend(extract_cids(item))
            elif asset is not None:
                cids.append(asset.cid)
            return cids

        result_asset = self.__results_to_assets__(result)
        output_cids = extract_cids(result_asset)

        compute_statement_ids = add_computation_statement(
            inputs=input_cids,
            outputs=output_cids,
            computation=None,
            _skip_proof=self._skip_proof,
            context=self._ctx,
        )

        self.metadata.create_statement(
            compute_statement_ids[0], self._skip_proof, context=self._ctx
        )

        return output_cids
