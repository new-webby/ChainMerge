from __future__ import annotations

import json
from typing import Any, Callable, Mapping
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode
from urllib.request import Request, urlopen

from .errors import ChainMergeAPIError, ChainMergeTransportError
from .types import ExampleTx, HealthResponse, NormalizedTransaction, SUPPORTED_CHAINS

Transport = Callable[[str, Mapping[str, str], float], tuple[int, str]]


def _default_transport(url: str, headers: Mapping[str, str], timeout: float) -> tuple[int, str]:
    request = Request(url=url, headers=dict(headers), method="GET")

    try:
        with urlopen(request, timeout=timeout) as response:
            status_code = int(response.getcode() or 200)
            body = response.read().decode("utf-8")
            return status_code, body
    except HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        return int(exc.code), body
    except URLError as exc:
        raise ChainMergeTransportError(f"request failed: {exc.reason}") from exc


class ChainMergeClient:
    """Client for the ChainMerge /api/decode endpoint."""

    def __init__(
        self,
        *,
        base_url: str = "https://chainmerge-api.onrender.com",
        api_key: str | None = None,
        timeout: float = 15.0,
        transport: Transport | None = None,
    ) -> None:
        if not base_url or not base_url.strip():
            base_url = "https://chainmerge-api.onrender.com"

        self._base_url = base_url.rstrip("/")
        self._api_key = api_key
        self._timeout = timeout
        self._transport = transport or _default_transport

    @property
    def base_url(self) -> str:
        return self._base_url

    def health(self) -> HealthResponse:
        """Check API health."""
        status_code, body = self._call_api("/api/health")
        payload = _load_json(body)
        if status_code >= 400:
            raise _api_error_from_response(status_code, payload, body)
        return HealthResponse.from_dict(payload)

    def examples(self) -> list[ExampleTx]:
        """Get example transactions supported by the API."""
        status_code, body = self._call_api("/api/examples")
        payload = _load_json(body)
        if status_code >= 400:
            raise _api_error_from_response(status_code, payload, body)
        raw_examples = payload.get("examples", [])
        return [ExampleTx.from_dict(item) for item in raw_examples]

    def get_metrics(self) -> str:
        """Get internal API metrics (Prometheus format string)."""
        status_code, body = self._call_api("/api/metrics")
        if status_code >= 400:
            payload = _load_json(body)
            raise _api_error_from_response(status_code, payload, body)
        return body

    def decode_tx(
        self,
        *,
        chain: str,
        tx_hash: str,
        rpc_url: str | None = None,
    ) -> NormalizedTransaction:
        params = self._build_params(chain, tx_hash, rpc_url)
        url = f"{self._base_url}/api/decode?{urlencode(params)}"
        return self._decode_at_url(url)

    def decode_and_index_tx(
        self,
        *,
        chain: str,
        tx_hash: str,
        rpc_url: str | None = None,
    ) -> NormalizedTransaction:
        """Decode a transaction and persist it in the backend index."""
        params = self._build_params(chain, tx_hash, rpc_url)
        url = f"{self._base_url}/api/index/decode?{urlencode(params)}"
        return self._decode_at_url(url)

    def lookup_indexed_tx(
        self,
        *,
        chain: str,
        tx_hash: str,
    ) -> NormalizedTransaction:
        """Lookup a previously decoded transaction from the backend index."""
        url = f"{self._base_url}/api/index/{chain}/{tx_hash}"
        return self._decode_at_url(url)

    def list_recent_indexed_txs(self, limit: int = 20) -> list[NormalizedTransaction]:
        """List recent transactions decoded and indexed by the API."""
        url = f"{self._base_url}/api/index/recent?limit={limit}"
        status_code, body = self._call_api_full_url(url)
        payload = _load_json(body)

        if status_code >= 400:
            raise _api_error_from_response(status_code, payload, body)

        raw_items = payload.get("items", [])
        return [NormalizedTransaction.from_dict(item) for item in raw_items]

    def decodeTx(
        self,
        *,
        chain: str,
        hash: str,
        rpcUrl: str | None = None,
    ) -> NormalizedTransaction:
        """JavaScript-style alias for decode_tx."""
        return self.decode_tx(chain=chain, tx_hash=hash, rpc_url=rpcUrl)

    def _call_api(self, path: str) -> tuple[int, str]:
        url = f"{self._base_url}{path}"
        return self._call_api_full_url(url)

    def _call_api_full_url(self, url: str) -> tuple[int, str]:
        headers: dict[str, str] = {"accept": "application/json"}
        if self._api_key:
            headers["x-api-key"] = self._api_key
        return self._transport(url, headers, self._timeout)

    def _build_params(self, chain: str, tx_hash: str, rpc_url: str | None) -> dict[str, str]:
        chain_key = chain.strip().lower()
        if chain_key not in SUPPORTED_CHAINS:
            supported = ", ".join(SUPPORTED_CHAINS)
            raise ValueError(f"Unsupported chain '{chain}'. Supported chains: {supported}")

        normalized_hash = tx_hash.strip()
        if not normalized_hash:
            raise ValueError("tx_hash is required")

        params: dict[str, str] = {
            "chain": chain_key,
            "hash": normalized_hash,
        }
        if rpc_url and rpc_url.strip():
            params["rpc_url"] = rpc_url.strip()
        return params

    def _decode_at_url(self, url: str) -> NormalizedTransaction:
        status_code, body = self._call_api_full_url(url)
        payload = _load_json(body)

        if status_code >= 400:
            raise _api_error_from_response(status_code, payload, body)

        decoded = payload.get("decoded")
        if not isinstance(decoded, dict):
            # Fallback for index lookup which might return NormalizedTransaction directly
            # check the payload shape
            if isinstance(payload, dict) and "chain" in payload and "tx_hash" in payload:
                return NormalizedTransaction.from_dict(payload)

            raise ChainMergeTransportError(
                "unexpected response payload: missing decoded object",
                status_code=status_code,
                raw=payload,
            )

        return NormalizedTransaction.from_dict(decoded)


def _load_json(body: str) -> Any:
    if not body:
        return {}
    try:
        return json.loads(body)
    except json.JSONDecodeError as exc:
        raise ChainMergeTransportError("invalid JSON response from ChainMerge API") from exc


def _api_error_from_response(
    status_code: int, payload: Any, body: str
) -> ChainMergeAPIError:
    if isinstance(payload, dict):
        envelope = payload.get("error")
        if isinstance(envelope, dict):
            message = str(
                envelope.get("message") or f"ChainMerge API request failed with HTTP {status_code}"
            )
            code_raw = envelope.get("code")
            code = str(code_raw) if code_raw is not None else None
            retryable_raw = envelope.get("retryable")
            retryable = retryable_raw if isinstance(retryable_raw, bool) else None
            return ChainMergeAPIError(
                message,
                status_code=status_code,
                code=code,
                retryable=retryable,
                raw=payload,
            )

        message = payload.get("message")
        if isinstance(message, str) and message.strip():
            return ChainMergeAPIError(
                message.strip(),
                status_code=status_code,
                raw=payload,
            )

    fallback = f"ChainMerge API request failed with HTTP {status_code}"
    if body.strip():
        fallback = body.strip()[:400]

    return ChainMergeAPIError(
        fallback,
        status_code=status_code,
        raw=payload,
    )

