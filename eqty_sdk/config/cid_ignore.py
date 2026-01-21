from pydantic import BaseModel


class CidIgnore(BaseModel):
    """Configuration for file inclusion settings when CIDing a directory."""

    include_hidden_files: bool = False
    gitignore: bool = False
    include_symlinks: bool = False
