from __future__ import annotations

from typing import Any


class ChainMergeError(Exception):
    """Base SDK error."""

    def __init__(
        self,
        message: str,
        *,
        status_code: int | None = None,
        code: str | None = None,
        retryable: bool | None = None,
        raw: Any = None,
    ) -> None:
        super().__init__(message)
        self.status_code = status_code
        self.code = code
        self.retryable = retryable
        self.raw = raw


class ChainMergeAPIError(ChainMergeError):
    """Raised for non-2xx API responses."""


class ChainMergeTransportError(ChainMergeError):
    """Raised for network failures or invalid API payloads."""


