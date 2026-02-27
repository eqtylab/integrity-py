__all__ = [
    "Error",
    "ExternalError",
    "AuthenticationError",
    "UsageError",
    "UnsupportedError",
]

from typing import Optional


class Error(Exception):
    """Base Eqty SDK Error."""

    def __init__(self, message, context: Optional[dict] = None) -> None:
        super().__init__(message)
        self.message = message
        if context:
            self.context = context


class ExternalError(Error):
    """Error communicating with external servers."""

    def __init__(self, msg, exc=None) -> None:
        self.exc = exc
        self.message = msg
        super().__init__(self.message)


class AuthenticationError(ExternalError):
    """Raised when authentication fails."""


class UsageError(Error):
    """Raised when an invalid usage of the Eqty SDK is detected."""


class UnsupportedError(UsageError):
    """Raised when trying to use a feature that is not supported."""
