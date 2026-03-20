# Async Compute

The compute decorator works with both async functions and async generators.

Source: `examples/async.py`

```python
import asyncio
from eqty_sdk import Dataset, Signer, compute, init, set_active_signer

cfg = init()
signer = Signer.new()
set_active_signer(signer)

model_url = Dataset.from_object("https://huggingface.co/example", name="Model URL")


@compute(metadata={"name": "Async Function"})
async def compute_download_async(url: Dataset):
    output_model = f"downloaded bytes of {url.value}"
    await asyncio.sleep(1)
    return Dataset.from_object(output_model, name="Downloaded model")


@compute(metadata={"name": "Generator Function"})
async def compute_download_gen(url: Dataset):
    yield "This "
    await asyncio.sleep(0.2)
    yield "is "
    yield "the "
    await asyncio.sleep(0.3)
    yield "stream "
```

The key distinction:

- async functions finalize once they return a value
- async generators stream chunks and finalize when the generator completes
