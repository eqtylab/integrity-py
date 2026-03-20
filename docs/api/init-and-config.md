# Init And Config

`init()` is the required entry point for the SDK. It initializes the local SDK state on disk and returns a `Config` object that you can use to adjust hashing, blob storage, and context behavior.

## Initialize The SDK

The simplest form uses the default SDK directory under the current working directory:

```python
from eqty_sdk import init

cfg = init()
```

You can also provide:

- `default_context`: the context used when no context is passed explicitly to other functions
- `custom_dir`: a custom SDK working directory

```python
from pathlib import Path

from eqty_sdk import Context, init

# set a user friendly name on the context
ctx = Context.new("customer-project")
cfg = init(default_context=ctx, custom_dir=Path(".eqty_sdk_customer"))
```

## What `init()` Returns

`init()` returns a `Config` object. The main public methods on that object are:

- `set_hashing_config(...)`
- `set_cid_ignore_rules(...)`
- `set_store_all_blobs(...)`
- `get_default_context()`

## Configure Hashing

Use `set_hashing_config()` to control hashing behavior when computing CIDs.

```python
from eqty_sdk import init

cfg = init()
cfg = cfg.set_hashing_config(multithread=True, memory_map=True)
```

Arguments:

- `multithread`: enable multithreaded hashing
- `memory_map`: enable memory-mapped file reads where supported

## Configure Directory Ignore Rules

Use `set_cid_ignore_rules()` to control which files are included when hashing directories.

```python
from eqty_sdk import init

cfg = init()
cfg = cfg.set_cid_ignore_rules(
    include_hidden_files=False,
    gitignore=True,
    include_symlinks=False,
)
```

Arguments:

- `include_hidden_files`: include dotfiles and hidden paths
- `gitignore`: respect `.gitignore` patterns
- `include_symlinks`: include symlinks in directory hashing

## Store All Blobs Automatically

Use `set_store_all_blobs()` to persist blobs automatically when CIDs are computed.

```python
from eqty_sdk import init

cfg = init()
cfg = cfg.set_store_all_blobs(True)
```

This is useful when you expect to export manifests or register blobs with a remote service later.

## Read The Default Context

Use `get_default_context()` to inspect or export the context that `init()` is using by default.

```python
from pathlib import Path

from eqty_sdk import init

cfg = init()
ctx = cfg.get_default_context()
ctx.export(Path("default-context.json"))
```

## Full Example

```python
from pathlib import Path

from eqty_sdk import Context, Signer, init, set_active_signer

default_ctx = Context.new("customer-project")
cfg = init(default_context=default_ctx, custom_dir=Path(".eqty_sdk_customer"))

cfg = cfg.set_hashing_config(multithread=True, memory_map=True)
cfg = cfg.set_cid_ignore_rules(
    include_hidden_files=False,
    gitignore=True,
    include_symlinks=False,
)
cfg = cfg.set_store_all_blobs(True)

signer = Signer.new()
set_active_signer(signer)

ctx = cfg.get_default_context()
print(ctx)
```
