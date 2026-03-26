# Install From The Eqty Package Index

Install `eqty_sdk` from the Eqty package index:

`https://pypi.eqtylab.io/simple/`

The examples below use that index explicitly so the package manager resolves `eqty_sdk` from the correct repository.

## pip

```bash
python -m pip install --index-url https://pypi.eqtylab.io/simple/ eqty_sdk
```

## uv

```bash
uv pip install --index-url https://pypi.eqtylab.io/simple/ eqty_sdk
```

## Poetry

```bash
poetry source add --priority=explicit eqty https://pypi.eqtylab.io/simple/
poetry add --source eqty eqty_sdk
```
