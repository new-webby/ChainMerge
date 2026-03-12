# ChainMerge One-Pager

## What ChainMerge Is

ChainMerge is a unified multichain transaction decoding platform.

Input:
- `chain`
- `transaction_hash`
- `rpc_url`

Output:
- one normalized JSON schema (`token_transfer`-style events) regardless of source chain format.

## Why It Matters

Multichain builders currently maintain separate parsers for each network. ChainMerge removes that duplication by standardizing decode output, so wallets, analytics platforms, and cross-chain apps can integrate once.

## What Works Today

### Supported chains (implemented)
- `solana` (SPL transfer, transferChecked)
- `ethereum` (ERC-20 transfer + native ETH transfer)
- `cosmos` (bank MsgSend)
- `aptos` (aptos_account::transfer path)
- `sui` (balance-change transfer inference)
- `bitcoin` (UTXO tx transfer extraction)
- `polkadot` (balances transfer extraction)
- `starknet` (transfer-shaped receipt event parsing)

### API surface
- `GET /api/health`
- `GET /api/examples`
- `GET /api/metrics`
- `GET /api/decode`
- `GET /api/index/decode`
- `GET /api/index/{chain}/{hash}`
- `GET /api/index/recent?limit=<n>`

### Platform controls
- optional API key auth (`API_KEY`, header `x-api-key`)
- rate limiting (`RATE_LIMIT_PER_MIN`)
- persistent SQLite decode index (`INDEX_DB_PATH`)
- retry/failover RPC helper with weighted endpoint format and circuit-open backoff
- CI + Docker deployment included

## Architecture Snapshot

- Core engine: Rust (`core/chainmerge`)
- API: Rust + Axum (`services/api`)
- Frontend demo: React + Vite (`apps/web`)
- Indexing: SQLite-backed transaction index

## Compliance With Initial Vision

### ChainMerge Lite / hackathon scope
- Status: `Complete`

### Full ChainMerge + ChainKit in this repository scope
- Status: `Implemented end-to-end`
- Includes:
  - unified decoder core
  - multichain coverage
  - API layer
  - reliability controls (`chainrpc`)
  - canonicalized error model (`chainerrors`)
  - persistent index API (`chainindex` implementation path)

## Proof Commands

```bash
make test
```

```bash
curl "http://127.0.0.1:8080/api/health"
```

```bash
curl "http://127.0.0.1:8080/api/examples"
```

```bash
curl "http://127.0.0.1:8080/api/decode?chain=ethereum&hash=0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad&rpc_url=<ETH_RPC_URL>"
```

## Business-Ready Positioning

ChainMerge can now be presented as:
- a production-capable multichain decoding API
- a foundation layer for wallets, analytics, and cross-chain apps
- a deployable system with test, CI, and containerized operations already in place
