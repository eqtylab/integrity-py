import asyncio
import logging
import os
from datetime import datetime
from pathlib import Path

from eqty_sdk import (
    SIGNER_ALGORITHMS,
    Asset,
    Dataset,
    Signer,
    compute,
    init,
    set_active_signer,
)

##############################################
###       General logging setup            ###
##############################################
logging.basicConfig(
    level=logging.DEBUG,
    format="(%(asctime)s) %(levelname)s - %(name)s %(funcName)s: %(message)s",
    handlers=[logging.StreamHandler()],
)

asyncio_logger = logging.getLogger("asyncio")
asyncio_logger.setLevel(logging.WARNING)

sdk_logger = logging.getLogger("eqty.sdk")
sdk_logger.setLevel(logging.DEBUG)

##############################################
###              SDK usage                 ###
##############################################

cfg = init()
signer = Signer.from_private_key(
    algorithm=SIGNER_ALGORITHMS.ED25519,
    private_key="eHb22WNFvUXihogn8fubQjW7hHEqwY3fEKt745V4xXg=",
)
set_active_signer(signer)

model_url = Dataset.from_object("https://huggingface.co/example", name="Model URL")
model_url2 = Dataset.from_object("http://huggingface.co/example", name="Model-URL 2")

time_now = datetime.now().strftime("%H:%M")


# Decorating an Async Function
@compute(metadata={"name": "Async Function"})
async def compute_download_async(url: Asset):
    output_model = f"downloaded bytes of {url.value}"
    await asyncio.sleep(1)

    dataset = Dataset.from_object(
        output_model, name="Downloaded model", description="Downloaded model from Huggingface"
    )
    return dataset


# Decorating an Async Generator Function
@compute(
    metadata={
        "name": "Generator Function",
        "description": "async function that generates data",
    }
)
async def compute_download_gen(url: Asset):
    """Async generator decorated fn.
    This doc string should be in the metadata.
    """
    yield "This "
    await asyncio.sleep(0.2)
    yield "is "
    yield "the "
    await asyncio.sleep(0.3)
    yield "stream "


async def main():
    # Enable/disable different computes:
    async for y in compute_download_gen(model_url):
        logging.info(y)
    # await compute_download_async(model_url2)


asyncio.run(main())

name_str = "gen"
ctx = cfg.get_default_context()
ctx.export(Path(f"{name_str}.json"))
