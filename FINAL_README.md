# ChainMerge Final README

ChainMerge is a multichain transaction decoder.
It accepts chain-specific transaction identifiers and returns one normalized response shape.

## Overview

Problem solved:
- Multichain apps usually maintain separate parser logic per chain.

What ChainMerge does:
- Decodes chain-specific transaction payloads.
- Normalizes outputs into a common schema.
- Exposes a single HTTP API for decode and indexed lookup.
- Provides a web UI for quick testing.

Core value:
- Integrate once at API/schema level, not once per chain format.

## What This Project Can Do

- Decode transfer-focused transaction patterns for:
  - `ethereum`
  - `solana`
  - `cosmos`
  - `aptos`
  - `sui`
  - `polkadot`
  - `bitcoin`
  - `starknet`
- Return standardized JSON with common fields:
  - `chain`
  - `tx_hash`
  - `sender`
  - `receiver`
  - `value`
  - `events[]`
- Expose health/examples/metrics endpoints.
- Persist decoded transactions into a SQLite index.
- Lookup indexed transactions by chain+hash.
- Query recent indexed transactions.

## Architecture

### 1) Core Decoder (`core/chainmerge`)

- Language: Rust
- Role:
  - chain-specific fetch + decode
  - normalization into shared schema
  - deterministic parser behavior
- Key components:
  - `src/chains/*`: chain decoder modules
  - `src/types/mod.rs`: schema + validation
  - `src/errors.rs`: canonical error model
  - `src/chainrpc/mod.rs`: weighted failover helper

### 2) Backend API (`services/api`)

- Language: Rust + Axum
- Role:
  - request parsing and validation
  - chain dispatch into core decoder
  - HTTP error mapping
  - metrics, rate limiting, auth guard
  - SQLite index persistence and query
- Default decode behavior:
  - `rpc_url` query is optional
  - backend resolves per-chain default RPC URLs internally
  - request can still override via `rpc_url`

### 3) Frontend UI (`apps/web`)

- Language: React + TypeScript + Vite
- Role:
  - select chain
  - enter transaction hash
  - optional RPC override input
  - show decoded output/error JSON

### 4) Deployment/Ops

- Docker support:
  - `services/api/Dockerfile`
  - `apps/web/Dockerfile`
  - `docker-compose.yml`
- Makefile shortcuts for dev/test/run

## API Endpoints

- `GET /api/health`
- `GET /api/examples`
- `GET /api/metrics`
- `GET /api/decode?chain=<chain>&hash=<tx>&rpc_url=<optional>`
- `GET /api/index/decode?chain=<chain>&hash=<tx>&rpc_url=<optional>`
- `GET /api/index/{chain}/{hash}`
- `GET /api/index/recent?limit=<n>`

## Project Structure

```txt
.
├── apps/
│   └── web/
│       ├── src/
│       ├── package.json
│       └── vite.config.ts
├── core/
│   └── chainmerge/
│       ├── src/
│       │   ├── chains/
│       │   ├── chainrpc/
│       │   ├── errors.rs
│       │   ├── traits/
│       │   └── types/
│       ├── tests/
│       └── Cargo.toml
├── services/
│   └── api/
│       ├── src/main.rs
│       └── Cargo.toml
├── docker-compose.yml
├── Makefile
├── STARTUP_README.md
├── USAGE_README.md
└── rpcURLS.MD
```

## Supported RPC Types by Chain

- `ethereum`: JSON-RPC
- `solana`: JSON-RPC
- `cosmos`: REST/LCD
- `aptos`: REST
- `sui`: JSON-RPC
- `bitcoin`: Blockstream-compatible REST
- `polkadot`: Subscan API (current decoder path)
- `starknet`: JSON-RPC

Reference list:
- `rpcURLS.MD`

## Local Startup

From repo root:

Terminal 1:

```bash
make run-api
```

Terminal 2:

```bash
make run-web
```

Open:
- UI: `http://127.0.0.1:5173`
- API: `http://127.0.0.1:8080`

## Important Runtime Environment Variables

- `HOST`
- `PORT`
- `API_KEY`
- `RATE_LIMIT_PER_MIN`
- `INDEX_DB_PATH`
- `POLKADOT_SUBSCAN_API_KEY`
- `CHAINMERGE_RPC_URL` (global override)
- `CHAINMERGE_RPC_URL_ETHEREUM`
- `CHAINMERGE_RPC_URL_SOLANA`
- `CHAINMERGE_RPC_URL_COSMOS`
- `CHAINMERGE_RPC_URL_APTOS`
- `CHAINMERGE_RPC_URL_SUI`
- `CHAINMERGE_RPC_URL_POLKADOT`
- `CHAINMERGE_RPC_URL_BITCOIN`
- `CHAINMERGE_RPC_URL_STARKNET`

## Known Scope and Limits

- Decoder focus is transfer-shaped events, not full protocol semantics.
- `polkadot` currently depends on Subscan response model.
- Some valid transactions may return `unsupported_event` if they are non-transfer actions.
- RPC/provider quality affects decode reliability.

## Next Practical Enhancements

1. Add Polkadot node JSON-RPC decoding path (reduce Subscan dependency).
2. Add multi-event output (return more than first matched event).
3. Extend coverage to swaps, staking, approvals, and contract-call intents.
4. Add richer fixtures and integration tests per chain/provider.
5. Add OpenAPI spec + typed SDKs.

## Additional Docs

- Usage: `USAGE_README.md`
- Startup: `STARTUP_README.md`
- RPC URLs: `rpcURLS.MD`
- Progress report: `REPORT_README.md`
