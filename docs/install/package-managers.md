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

## PDM

```bash
pdm add --index https://pypi.eqtylab.io/simple/ eqty_sdk
```

## Pipenv

```bash
pipenv install --index https://pypi.eqtylab.io/simple/ eqty_sdk
```

## Conda

Create or activate the conda environment first, then use `pip` inside that environment:

```bash
conda activate <env-name>
python -m pip install --index-url https://pypi.eqtylab.io/simple/ eqty_sdk
```

## Mamba

`mamba` manages the environment in the same way as `conda`, so install the package with `pip` after activating the environment:

```bash
mamba activate <env-name>
python -m pip install --index-url https://pypi.eqtylab.io/simple/ eqty_sdk
```

## Pixi

If you are using a Pixi-managed environment, install the package with `pip` inside that environment:

```bash
pixi run python -m pip install --index-url https://pypi.eqtylab.io/simple/ eqty_sdk
```
