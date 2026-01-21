import asyncio
from typing import Any, Callable

import nest_asyncio


def run_async_helper(func: Callable, *args: Any, **kwargs: Any) -> Any:
    try:
        loop = asyncio.get_running_loop()
    except RuntimeError:
        loop = None

    if loop and loop.is_running():
        nest_asyncio.apply()
        return loop.run_until_complete(func(*args, **kwargs))
    else:
        return asyncio.run(func(*args, **kwargs))
