__all__ = [
    "Error",
    "UsageError",
]

from typing import Optional


class Error(Exception):
    """Base Eqty SDK Error."""

    def __init__(self, message, context: Optional[dict] = None) -> None:
        super().__init__(message)
        self.message = message
        if context:
            self.context = context


class UsageError(Error):
    """Raised when an invalid usage of the Eqty SDK is detected."""
