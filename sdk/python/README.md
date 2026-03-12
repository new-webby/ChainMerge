# chaincodec-sdk (Python)

Python SDK for the ChainCodec multichain transaction decoder API.

ChainCodec normalizes transactions from multiple chains (Ethereum, Solana, Cosmos, Aptos, Sui, Polkadot, Bitcoin, Starknet) into a single JSON shape.

## Installation

```bash
pip install chaincodec-sdk
```

For local development from this repository:

```bash
cd sdk/python
python3 -m pip install -e .
```

## Quick Start

```python
from chaincodec_sdk import ChainCodecClient

client = ChainCodecClient(
    base_url="http://127.0.0.1:8080",
    # api_key="optional-api-key",
)

tx = client.decode_tx(
    chain="ethereum",
    tx_hash="0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad",
)

print(tx.chain, tx.tx_hash)
for event in tx.events:
    if event.event_type == "token_transfer":
        print(event.token, event.amount, event.from_address, event.to_address)
```

## API

### `ChainCodecClient`

```python
ChainCodecClient(
    base_url: str,
    api_key: str | None = None,
    timeout: float = 15.0,
)
```

- `base_url`: ChainCodec API base URL (`http://127.0.0.1:8080`, etc.).
- `api_key`: Optional API key sent as `x-api-key`.
- `timeout`: Request timeout in seconds.

### `client.decode_tx(...)`

```python
decode_tx(chain: str, tx_hash: str, rpc_url: str | None = None) -> NormalizedTransaction
```

- `chain`: `solana|ethereum|cosmos|aptos|sui|polkadot|bitcoin|starknet`
- `tx_hash`: transaction hash string
- `rpc_url`: optional per-request RPC URL override

### Errors

Non-2xx API responses raise `ChainCodecAPIError` with:
- `status_code`
- `code` (backend error code, when present)
- `retryable` (when present)

Network and JSON decode issues raise `ChainCodecTransportError`.

## Publishing

From `sdk/python`:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade build twine
python -m unittest discover -s tests -v
python -m build
python -m twine check dist/*
python -m twine upload dist/*
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
