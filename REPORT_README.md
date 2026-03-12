# ChainMerge Progress Report

## Project Status (as of now)

### Phase 0: Contracts First (Completed)
Implemented in `core/chainmerge`:
- Canonical chain enum (`Chain`) for supported networks
- Universal transaction/event schema
- Decode request validation (`rpc_url`, tx hash format checks)
- Standardized error model + error envelope (`ErrorCode`, `ErrorEnvelope`)
- Decoder dispatch via typed chain matching

Key files:
- `core/chainmerge/src/types/mod.rs`
- `core/chainmerge/src/errors.rs`
- `core/chainmerge/src/lib.rs`

### Phase 1: Solana MVP Decoder (Completed)
Implemented in `core/chainmerge/src/chains/solana/mod.rs`:
- Solana RPC call using `getTransaction` with `jsonParsed`
- Parses both outer instructions and inner instructions
- Supports SPL token events:
  - `transfer`
  - `transferChecked` (Token-2022 included)
- Normalizes into universal `token_transfer` event output
- Returns structured `UnsupportedEvent` when no supported transfer exists

### Phase 2: Rust Backend API (Completed)
Implemented in `services/api`:
- Rust backend with `axum`
- Endpoint:
  - `GET /api/health`
  - `GET /api/examples`
  - `GET /api/metrics`
  - `GET /api/decode?chain=<chain>&hash=<tx>&rpc_url=<url>`
  - `GET /api/index/decode?chain=<chain>&hash=<tx>&rpc_url=<url>`
  - `GET /api/index/{chain}/{hash}`
  - `GET /api/index/recent?limit=<n>`
- Direct integration with core crate:
  - `chainmerge::decode_transaction(...)`
- Structured error responses mapped to HTTP status codes
- Request controls:
  - optional API key auth (`API_KEY` + `x-api-key`)
  - rate limiting (`RATE_LIMIT_PER_MIN`)
  - request/decode/index counters
  - persistent decode index (`INDEX_DB_PATH`, SQLite)

Key files:
- `services/api/Cargo.toml`
- `services/api/src/main.rs`

### Phase 3: Ethereum ERC-20 Decoder (Completed)
Implemented in `core/chainmerge/src/chains/ethereum/mod.rs`:
- Ethereum RPC call via `eth_getTransactionReceipt`
- Parses receipt logs for ERC-20 `Transfer` topic
- Extracts token contract, from, to, and uint256 amount
- Normalizes to `token_transfer` events
- Fallback support for native ETH transfers via `eth_getTransactionByHash`
  - maps to normalized `token_transfer` with `token=ETH`
- Returns `UnsupportedEvent` only when neither ERC-20 nor native transfer is found

### Phase 3.5: Extended Chain Decoder Coverage (Completed)
Implemented practical transfer decoders for previously placeholder chains:
- Aptos: `0x1::aptos_account::transfer` payload parsing
- Sui: transfer inference from `balanceChanges`
- Bitcoin: Blockstream-style `/tx/{hash}` parsing
- Polkadot: Subscan-style balances transfer parsing
- Starknet: transfer-shaped receipt event parsing

Live proof (verified):
- Hash:
  - `0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad`
- API response validated:
  - `chain=ethereum`
  - event type `token_transfer`
  - `from`, `to`, `token`, `amount` correctly parsed

## Currently Supported Chain Keys

These are accepted by the core dispatch:
- `solana`
- `ethereum`
- `cosmos`
- `aptos`
- `sui`
- `polkadot`
- `bitcoin`
- `starknet`

Note:
- Real decoding paths now exist for all declared chain keys.
- Quality of decode signals varies by provider and available RPC response richness.

## Test Coverage Added

### Core (`core/chainmerge`)
- Chain parsing tests
- Request validation tests
- Solana parser tests:
  - SPL transfer extraction
  - transferChecked extraction
  - non-token instruction ignored
- Ethereum parser tests:
  - ERC-20 transfer extraction
  - non-transfer topic ignored
  - topic address extraction
  - hex-to-decimal amount conversion
- Aptos parser test:
  - transfer payload extraction
- Sui parser test:
  - balance-change transfer pairing
- Bitcoin parser test:
  - vin/vout transfer extraction
- Polkadot parser test:
  - Subscan balances transfer extraction
- Starknet parser test:
  - transfer-shaped event extraction

### API (`services/api`)
- Unknown chain -> `400`
- Missing `rpc_url` (when env var not set) -> `400`
- Invalid Ethereum hash -> `422`
- Health endpoint -> `200`
- Examples endpoint -> `200`
- Metrics endpoint -> `200`
- API key guard -> `401` when key missing/invalid
- Rate limit guard -> `429` after configured threshold

### Fixture-Based Decoder Tests
- Ethereum fixture:
  - `core/chainmerge/tests/fixtures/ethereum/erc20_receipt.json`
- Cosmos fixture:
  - `core/chainmerge/tests/fixtures/cosmos/msg_send_tx.json`
- Purpose:
  - validate parser behavior without live network dependency

## How to Test

## 1) Run Core Tests

```bash
cd /home/escanor/donda/core/chainmerge
cargo test
```

Expected:
- all core tests pass (Solana + Ethereum parser coverage)

## 2) Run API Tests

```bash
cd /home/escanor/donda/services/api
cargo test
```

Expected:
- all API route/error tests pass

## 3) Run API Service Locally

```bash
cd /home/escanor/donda/services/api
cargo run
```

Default server:
- `http://0.0.0.0:8080`

Optional env:
- `HOST` (default `0.0.0.0`)
- `PORT` (default `8080`)
- `CHAINMERGE_RPC_URL` (used if `rpc_url` query is not provided)

## 4) Test Decode Endpoint (Ethereum success case)

```bash
curl "http://127.0.0.1:8080/api/decode?chain=ethereum&hash=0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad&rpc_url=<ETH_RPC_URL>"
```

Expected:
- success JSON with `decoded.events[0].event_type = token_transfer`

## 4.1) Test Decode Endpoint (Cosmos success case)

Verified Cosmos Hub `MsgSend` hash:
- `6C166D13D94E626BB6477398B1D0AEB9B5C595D0B0DA8FC7AD2191BEEF024A27`

Verified LCD base URL:
- `https://cosmos-rest.publicnode.com`

```bash
curl "http://127.0.0.1:8080/api/decode?chain=cosmos&hash=6C166D13D94E626BB6477398B1D0AEB9B5C595D0B0DA8FC7AD2191BEEF024A27&rpc_url=https://cosmos-rest.publicnode.com"
```

Expected:
- success JSON with `decoded.events[*].event_type = token_transfer`
- `raw_program = cosmos_bank`

## 5) Quick Negative Checks

Unknown chain:

```bash
curl -i "http://127.0.0.1:8080/api/decode?chain=unknown&hash=abc&rpc_url=https://rpc.example"
```

Native ETH transfer decode success case:

```bash
curl "http://127.0.0.1:8080/api/decode?chain=ethereum&hash=0x5db45209923531658781b4a5ea73bde7193e7f0991595ad5af80121764afb8b4&rpc_url=<ETH_RPC_URL>"
```

Health check:

```bash
curl "http://127.0.0.1:8080/api/health"
```

Examples list:

```bash
curl "http://127.0.0.1:8080/api/examples"
```

### Phase 4: UI + Demo Readiness (MVP Completed)
Implemented in `apps/web`:
- React + Vite demo app scaffold
- Decode form:
  - chain selector
  - tx hash input
  - optional RPC URL input
- Calls backend endpoint `/api/decode`
- Displays formatted JSON success/error output
- Includes quick preset buttons for verified Ethereum examples:
  - ERC-20 transfer preset
  - native ETH transfer preset
- Includes local Vite proxy (`/api` -> `http://127.0.0.1:8080`) for easy dev

Supporting backend update:
- Added CORS layer in Rust API for browser requests during demo

Key files:
- `apps/web/package.json`
- `apps/web/vite.config.ts`
- `apps/web/src/App.tsx`
- `apps/web/src/main.tsx`
- `apps/web/src/styles.css`
- `services/api/src/main.rs`

## Frontend Run Steps (Phase 4)

## 6) Run Frontend Demo

```bash
cd /home/escanor/donda/apps/web
npm install
npm run dev
```

Open:
- `http://127.0.0.1:5173`

Requirements:
- backend API running on `http://127.0.0.1:8080`

## Next Planned Work

- Add provider-specific integration tests per chain with recorded fixtures
- Add durable indexing backend (DB + replay/recovery) beyond in-memory index
- Expand chainerrors taxonomy with richer per-chain signatures/codes
- Add observability export integration (Prometheus/OpenTelemetry)
