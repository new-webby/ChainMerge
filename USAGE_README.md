# ChainCodec Usage Guide

This guide explains how to run ChainCodec and how to call it from UI or API.

## What ChainCodec Does

Input:
- `chain`
- `hash` (transaction hash/digest)
- `rpc_url` (optional override; backend has built-in per-chain defaults)

Output:
- one normalized JSON response schema across supported chains.

## Supported Chains

- `solana`
- `ethereum`
- `cosmos`
- `aptos`
- `sui`
- `polkadot`
- `bitcoin`
- `starknet`

## RPC URL Expectations

- `ethereum`: EVM JSON-RPC endpoint
- `solana`: Solana JSON-RPC endpoint
- `cosmos`: Cosmos REST/LCD base URL
- `aptos`: Aptos fullnode REST base URL
- `sui`: Sui JSON-RPC endpoint
- `bitcoin`: Blockstream-compatible REST base URL
- `polkadot`: Subscan API base URL (`https://polkadot.api.subscan.io`)
- `starknet`: Starknet JSON-RPC endpoint

Note:
- Polkadot decode currently uses Subscan API shape, not node JSON-RPC.
- Subscan typically requires `POLKADOT_SUBSCAN_API_KEY`.

## Quick Start

Use the startup guide in [STARTUP_README.md](STARTUP_README.md).
Working RPC URL list: [rpcURLS.MD](rpcURLS.MD).

## Use Through the Web App

1. Start API and web app (see startup guide).
2. Open `http://127.0.0.1:5173`.
3. Select a chain.
4. Enter tx hash.
5. Optionally set RPC URL override.
6. Click `Decode Transaction`.

## Use Through the API

Health:

```bash
curl "http://127.0.0.1:8080/api/health"
```

Examples:

```bash
curl "http://127.0.0.1:8080/api/examples"
```

Decode:

```bash
curl "http://127.0.0.1:8080/api/decode?chain=ethereum&hash=0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad"
```

Decode + index:

```bash
curl "http://127.0.0.1:8080/api/index/decode?chain=ethereum&hash=0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad"
```

Lookup indexed tx:

```bash
curl "http://127.0.0.1:8080/api/index/ethereum/0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad"
```

Recent indexed txs:

```bash
curl "http://127.0.0.1:8080/api/index/recent?limit=20"
```

## Use Through the JavaScript/TypeScript SDK

For dapps, wallets, or backend services, you can use the published npm package instead of calling the HTTP API directly.

### Install

```bash
npm install chaincodec-sdk
# or
yarn add chaincodec-sdk
```

### Example (Node / browser)

```ts
import { ChainCodecClient } from "chaincodec-sdk";

const client = new ChainCodecClient({
  baseUrl: "http://127.0.0.1:8080", // or your hosted API URL
  // apiKey: "optional-x-api-key",
});

async function main() {
  const tx = await client.decodeTx({
    chain: "ethereum",
    hash: "0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad",
  });

  console.log("normalized tx:", tx);
}

main().catch((err) => {
  console.error("decode failed:", err);
});
```

## Use Through the Python SDK

For Python services, use the PyPI package:

### Install

```bash
pip install chaincodec-sdk
```

### Example

```python
from chaincodec_sdk import ChainCodecClient

client = ChainCodecClient(base_url="http://127.0.0.1:8080")

tx = client.decode_tx(
    chain="polkadot",
    tx_hash="0x65498725d9a093b63f9f312e5271926c8ac5e1493cba18428effd58dae2d2a6f",
)

print(tx.chain, tx.tx_hash, tx.value)
```

## Runtime Environment Variables

- `HOST`: API bind host (default `0.0.0.0`)
- `PORT`: API port (default `8080`)
- `CHAINCODEC_RPC_URL`: optional global RPC URL override when query `rpc_url` is omitted
- `CHAINCODEC_RPC_URL_ETHEREUM`: optional chain-specific override
- `CHAINCODEC_RPC_URL_SOLANA`: optional chain-specific override
- `CHAINCODEC_RPC_URL_COSMOS`: optional chain-specific override
- `CHAINCODEC_RPC_URL_APTOS`: optional chain-specific override
- `CHAINCODEC_RPC_URL_SUI`: optional chain-specific override
- `CHAINCODEC_RPC_URL_POLKADOT`: optional chain-specific override
- `CHAINCODEC_RPC_URL_BITCOIN`: optional chain-specific override
- `CHAINCODEC_RPC_URL_STARKNET`: optional chain-specific override
- `API_KEY`: if set, requests must send `x-api-key`
- `RATE_LIMIT_PER_MIN`: request limit per minute per key/client
- `INDEX_DB_PATH`: SQLite path for indexed decode storage
- `POLKADOT_SUBSCAN_API_KEY`: sent as `X-API-Key` to Subscan

## Common Errors

- `invalid_request: polkadot decoder expects a Subscan API base URL`
  - use `https://polkadot.api.subscan.io`, not `https://rpc.polkadot.io`
- `invalid_request: Subscan API key required`
  - set `POLKADOT_SUBSCAN_API_KEY`
- `invalid_transaction_hash`
  - hash format does not match chain rules
- `unsupported_event`
  - transaction parsed but no transfer-like event matched current decoder logic

## Enhancement Ideas

1. Add Polkadot node JSON-RPC decode path (`chain_getBlock`, event joins) to remove Subscan dependency.
2. Expand event model beyond transfer-like events (swap, mint, burn, stake, governance).
3. Return multiple decoded events per transaction instead of first matched transfer only.
4. Add richer chain-specific fixtures and live integration tests in CI.
5. Add response caching for repeated hash lookups and configurable cache TTL.
6. Add OpenAPI spec and typed SDK clients (Rust, Go).
7. Add background indexer mode (scan by block/checkpoint, not only request-time indexing).
8. Add observability improvements (structured metrics labels, tracing spans, error dashboards).
9. Add configurable per-chain timeouts and retry policies.
10. Add request ID + audit logging for easier debugging in production.
