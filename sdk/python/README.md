# chainmerge-sdk (Python)

Python SDK for the ChainMerge multichain transaction decoder API.

Use this SDK to call `GET /api/decode` and get a normalized transaction object across multiple chains.

## Requirements

- Python `3.9+`
- A running ChainMerge API instance (`base_url`)

## Install

From PyPI:

```bash
pip install chainmerge-sdk
```

From this repository (editable install):

```bash
cd sdk/python
python3 -m pip install -e .
```

## Quick Start

```python
from chainmerge_sdk import ChainMergeClient

client = ChainMergeClient(
    base_url="http://127.0.0.1:8080",
    # api_key="optional-api-key",
    timeout=15.0,
)

tx = client.decode_tx(
    chain="ethereum",
    tx_hash="0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad",
)

print("chain:", tx.chain)
print("tx_hash:", tx.tx_hash)
print("sender:", tx.sender)
print("receiver:", tx.receiver)

for action in tx.actions:
    print(action.action_type, action.token, action.amount)
```

## Use a Custom RPC URL

Pass `rpc_url` when you want to override the backend default RPC endpoint for a specific request.

```python
tx = client.decode_tx(
    chain="polkadot",
    tx_hash="0xyour_tx_hash",
    rpc_url="https://polkadot.api.subscan.io",
)
```

## Error Handling

```python
from chainmerge_sdk import (
    ChainMergeAPIError,
    ChainMergeClient,
    ChainMergeTransportError,
)

client = ChainMergeClient(base_url="http://127.0.0.1:8080")

try:
    tx = client.decode_tx(chain="ethereum", tx_hash="0x...")
except ChainMergeAPIError as err:
    # API returned non-2xx
    print("api error:", err)
    print("status_code:", err.status_code)
    print("code:", err.code)
    print("retryable:", err.retryable)
except ChainMergeTransportError as err:
    # Network issue or invalid JSON payload
    print("transport error:", err)
```

## Returned Data Shape

`decode_tx()` returns `NormalizedTransaction` with:

- `chain: str`
- `tx_hash: str`
- `sender: str | None`
- `receiver: str | None`
- `value: str | None`
- `events: list[NormalizedEvent]`
- `actions: list[Action]`

`NormalizedEvent` fields:

- `event_type`
- `token`
- `from_address`
- `to_address`
- `amount`
- `raw_program`

`Action` fields:

- `action_type`
- `from_address`
- `to_address`
- `amount`
- `token`
- `metadata`

## Supported Chains

- `solana`
- `ethereum`
- `cosmos`
- `aptos`
- `sui`
- `polkadot`
- `bitcoin`
- `starknet`

Passing an unsupported chain raises `ValueError`.

## API Reference

```python
from chainmerge_sdk import ChainMergeClient

client = ChainMergeClient(
    base_url: str,
    api_key: str | None = None,
    timeout: float = 15.0,
)

tx = client.decode_tx(
    chain: str,
    tx_hash: str,
    rpc_url: str | None = None,
)

# New 1.1.0 Methods:
client.health() -> HealthResponse
client.examples() -> ExamplesResponse
client.get_metrics() -> dict
client.decode_and_index_tx(chain, tx_hash, rpc_url=None) -> NormalizedTransaction
client.lookup_indexed_tx(chain, tx_hash) -> NormalizedTransaction
client.list_recent_indexed_txs(limit=10) -> list[NormalizedTransaction]
```

JavaScript-style alias is also available:

```python
tx = client.decodeTx(chain="ethereum", hash="0x...", rpcUrl=None)
```

## Local Development

From `sdk/python`:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -e .
python -m unittest discover -s tests -v
```

## GitHub Actions Publish

This repository includes `.github/workflows/publish-python-sdk.yml`:

- Push tag `python-sdk-vX.Y.Z` to publish to PyPI.
- Manual run (`workflow_dispatch`) supports `testpypi` or `pypi`.

Required repository secrets:

- `PYPI_API_TOKEN`
- `TEST_PYPI_API_TOKEN`

Tag release example:

```bash
git tag python-sdk-v0.1.0
git push origin python-sdk-v0.1.0
```

Manual local publish example (TestPyPI):

```bash
python -m twine upload --repository-url https://test.pypi.org/legacy/ dist/*
```

Manual local publish example (PyPI):

```bash
python3 -m build
python3 -m twine upload dist/*
```
