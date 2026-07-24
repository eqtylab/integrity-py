# Install From The Eqty Package Index

`eqty_sdk` is published to the Eqty package index, not to PyPI:

`https://pypi.eqtylab.io/simple/`

The index **requires authentication**, and it proxies PyPI: requests for packages Eqty
does not publish are forwarded upstream. That means you can point a package manager at
this index as its only index and still resolve `eqty_sdk`'s dependencies normally.

## Credentials

Every command on this page fails with `401 Unauthorized` until credentials are
configured. Depending on the package manager, the error surfaces as a bare
"not found in the package registry", so configure credentials first.

<!-- TODO(eqty): document how a user actually obtains an index username/token,
     and link it here. This is the single biggest gap in onboarding -- nothing on
     the docs site currently tells a new user where credentials come from. -->

Use a **read-only** token. Publishing is never required to install.

Never commit credentials. In particular, do not inline them into an index URL
(`https://user:token@pypi.eqtylab.io/simple/`) in `pyproject.toml` or `poetry.toml` —
those files are usually committed and often published inside source distributions.

### netrc (works with pip, uv, and Poetry)

```
machine pypi.eqtylab.io
  login <username>
  password <token>
```

### Environment variables

uv derives these names from the index name, so they match the index named `eqty` in the
`pyproject.toml` below:

```bash
export UV_INDEX_EQTY_USERNAME='<username>'
export UV_INDEX_EQTY_PASSWORD='<token>'
```

Poetry uses the source name added in the [Poetry](#poetry) section:

```bash
export POETRY_HTTP_BASIC_EQTY_USERNAME='<username>'
export POETRY_HTTP_BASIC_EQTY_PASSWORD='<token>'
```

pip has no equivalent environment variables; use `netrc` or a keyring.

### Keyring

For shared machines and CI, prefer a keyring backend over environment variables. uv and
pip both support `--keyring-provider subprocess`.

## uv (project)

This is the recommended setup for a uv project. It records `eqty_sdk` in
`pyproject.toml` and pins it to the Eqty index, so `uv sync` reproduces the environment
on any machine.

Add the index and the source pin to `pyproject.toml`:

```toml
[[tool.uv.index]]
name = "eqty"
url = "https://pypi.eqtylab.io/simple/"
explicit = true

[tool.uv.sources]
eqty-sdk = { index = "eqty" }
```

Then add the package:

```bash
uv add eqty-sdk
```

`explicit = true` means only packages pinned to this index in `[tool.uv.sources]`
resolve from it; everything else continues to come from PyPI. uv does not write
credentials into `pyproject.toml` or `uv.lock`, so both stay safe to commit.

## uv (no project)

To install into an activated environment without a `pyproject.toml`:

```bash
uv pip install --index-url https://pypi.eqtylab.io/simple/ eqty_sdk
```

Note that `uv pip install` does not record the dependency anywhere. In a uv project, a
later `uv sync` will remove `eqty_sdk` again — use the project setup above instead.

## pip

```bash
python -m pip install --index-url https://pypi.eqtylab.io/simple/ eqty_sdk
```

Use `--index-url`, not `--extra-index-url`. With `--extra-index-url`, pip searches PyPI
*and* the Eqty index and installs whichever has the highest version number, which is the
[dependency confusion](https://en.wikipedia.org/wiki/Supply_chain_attack) pattern: anyone
who uploads a higher-versioned `eqty_sdk` to PyPI would win. `--index-url` resolves from
the Eqty index only, and the index proxies PyPI for everything else.

## Poetry

```bash
poetry source add --priority=explicit eqty https://pypi.eqtylab.io/simple/
poetry add --source eqty eqty_sdk
```

`--priority=explicit` is the Poetry equivalent of uv's `explicit = true`: only packages
requested with `--source eqty` resolve from the Eqty index.

## Verify

```bash
python -c "import eqty_sdk; print(eqty_sdk.__file__)"
```

Next: [Basic Workflow](../examples/basic-workflow.md).
