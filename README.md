# ChainMerge Forensic: Gemini 3 API Challenge Submission

ChainMerge Forensic is a multichain transaction analyst that pivots from simple decoding into deep intent reasoning. Built for the **Gemini 3 API Challenge**, it leverages advanced AI to explain *why* transactions happen across 8+ different blockchain ecosystems.

## 🚀 Key Forensic Features
- **Forensic Analysis**: Powered by `gemini-1.5-flash` to detect intent (Arbitrage, Swaps, Phishing).
- **Transparency**: Displays the AI's "Thought Process" (reasoning) alongside the final report.
- **Multichain Forensic**: Normalizes raw data from Ethereum, Solana, Cosmos, Aptos, Sui, and more into readable narratives.
- **Whale Alert System**: Real-time identification of high-value transfers.

## Core Idea

Multichain apps should not rebuild parsing logic for every chain.  
`ChainMerge` converts chain-specific transaction formats into one universal JSON schema.  
`ChainKit` adds reliability, indexing, and error standardization around that core.

Input:
- `chain`
- `transaction_hash`

Output:
- normalized, chain-agnostic JSON

```json
{
  "chain": "solana",
  "tx_hash": "...",
  "sender": "...",
  "receiver": "...",
  "value": "...",
  "events": [
    {
      "type": "token_transfer",
      "token": "...",
      "from": "...",
      "to": "...",
      "amount": "..."
    }
  ]
}
```

## Architecture

### 1) Core Decode Layer (`core/chainmerge`)
- Language: Rust
- Responsibility:
  - fetch/decode chain transaction data
  - normalize outputs into universal schema
  - keep decode logic deterministic and testable
- Supported chain keys (current):
  - `solana`, `ethereum`, `cosmos`, `aptos`, `sui`, `polkadot`, `bitcoin`, `starknet`
- Current decode coverage:
  - `solana`: SPL `transfer` + `transferChecked`
  - `ethereum`: ERC-20 `Transfer` + native ETH transfer fallback
  - `cosmos`: `/cosmos.bank.v1beta1.MsgSend`
  - `aptos`: `0x1::aptos_account::transfer` payload parsing
  - `sui`: transfer inference from `balanceChanges`
  - `bitcoin`: Blockstream-compatible `/tx/{hash}` parsing
  - `polkadot`: Subscan-style `Balances.transfer*` parsing
  - `starknet`: transfer-shaped event parsing from receipt events
- Modules:
  - `traits/`: decoder trait contract
  - `chains/...`: chain-specific decoding modules
  - `normalizer/`: mapping chain outputs -> universal types
  - `types/`: canonical schema structs

### 2) API Layer (`services/api`)
- Language: Rust (`axum`)
- Responsibility:
  - expose decode endpoints
  - validate inputs and return structured error envelopes
  - call `chainmerge::decode_transaction` directly
- Core route:
  - `GET /api/health`
  - `GET /api/examples`
  - `GET /api/metrics`
  - `GET /api/decode?chain=<chain>&hash=<tx>&rpc_url=<url>`
  - `GET /api/index/decode?chain=<chain>&hash=<tx>&rpc_url=<url>`
  - `GET /api/index/{chain}/{hash}`
  - `GET /api/index/recent?limit=<n>`

### 3) Frontend Layer (`apps/web`)
- Language: React/TypeScript
- Responsibility:
  - chain + hash input
  - normalized JSON output viewer

### 4) Infra + Ops (`infra`)
- `docker/`: container definitions
- `scripts/`: local dev scripts, run/test helpers

## Repository Structure

```txt
.
├── core/
│   └── chainmerge/
│       └── src/
│           ├── traits/
│           ├── chains/
│           │   ├── solana/
│           │   ├── ethereum/
│           │   ├── cosmos/
│           │   ├── aptos/
│           │   ├── sui/
│           │   ├── polkadot/
│           │   ├── bitcoin/
│           │   └── starknet/
│           ├── normalizer/
│           ├── types/
│           └── lib.rs
├── services/
│   └── api/
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
├── apps/
│   └── web/
│       └── src/
│           ├── components/
│           ├── pages/
│           ├── lib/
│           └── main.tsx
├── infra/
│   ├── docker/
│   └── scripts/
├── docs/
├── flow.md
└── project_context.md
```

## Phase Pathway

### Phase 0: Contracts First
Goal: lock the canonical schema and interfaces.
- Define universal transaction schema in `core/chainmerge/src/types`
- Define decoder trait in `core/chainmerge/src/traits`
- Define error envelope for API responses

Deliverable:
- stable schema v0 and interface signatures

### Phase 1: Solana Track (MVP decode)
Goal: first working deterministic decoder.
- Implement Solana RPC fetch
- Decode SPL `Transfer` and `TransferChecked`
- Map into universal event format (`token_transfer`)
- Add tests with known transaction hashes

Deliverable:
- working `chain=solana` decode response

### Phase 2: API Bridge
Goal: make core decoder usable by apps.
- Build Rust HTTP API in `services/api` using `axum`
- Integrate API directly with `chainmerge` via local crate dependency
- Expose `GET /api/decode`
- Input validation and timeout handling

Deliverable:
- usable decode API endpoint for Solana

### Phase 3: Ethereum Track
Goal: prove chain-agnostic model works across ecosystems.
- Implement Ethereum decoder for ERC-20 `Transfer`
- Add native ETH transfer fallback
- Normalize into same output schema
- Reuse same API/UI with no schema changes

Deliverable:
- consistent output for Solana, Ethereum ERC-20, and Ethereum native transfers

### Phase 3.5: Cosmos Bank Track
Goal: add first non-EVM additional production decoder path.
- Implement Cosmos tx fetch (`/cosmos/tx/v1beta1/txs/{hash}`)
- Decode `/cosmos.bank.v1beta1.MsgSend`
- Normalize each coin movement into `token_transfer` events

Deliverable:
- working `chain=cosmos` decode response for bank sends

### Phase 3.6: Extended Chain Coverage
Goal: move all declared chain keys off placeholder-only behavior.
- Add practical transfer decode paths for Aptos, Sui, Bitcoin, Polkadot, Starknet
- Keep strict unsupported fallback when transfer semantics are not detected
- Add fixture/unit coverage per chain parser

Deliverable:
- transfer-path decoding available across all declared chain keys

### Phase 4: UI + Demo Readiness
Goal: present clear product value.
- Build form: chain + tx hash
- Show normalized JSON
- Curate demo transaction set and edge cases

Deliverable:
- demo-ready web app

### Phase 5: Reliability + Production Path
Goal: move from prototype to infrastructure layer.
- Add `chainrpc` failover logic
- Add `chainerrors` standardized error mapping
- Add `chainindex` for event stream/indexing

Deliverable:
- scalable path toward production ChainKit

Current status:
- `chainrpc`: retry/failover helper implemented
- `chainerrors`: canonical error domain + mapping utility implemented
- `chainindex`: in-memory API indexing/query implemented

## Immediate Build Start (Now)

1. Implement schema + trait skeleton in Rust (`Phase 0`).
2. Ship Solana decoder first (`Phase 1`).
3. Expose `GET /api/decode` with Solana support (`Phase 2`).
4. Add Ethereum decoder (`Phase 3`).

## Design Rules

- Decoder core remains deterministic and chain-correct.
- All chains map to the same schema, no exceptions in API contract.
- Unsupported events return structured errors, never crashes.

## Success Criteria

- Same output schema for Solana and Ethereum transfers
- End-to-end flow: decode -> display
- Stable error handling for invalid hash / timeout / unsupported event
- Live demo speed suitable for hackathon judging

## API Utility Endpoints

- `GET /api/health`: service health status
- `GET /api/examples`: ready-to-test transaction hashes for demo flows
- `GET /api/metrics`: request/decode/index counters
- `GET /api/index/{chain}/{hash}`: lookup decoded tx from in-memory index

## Deployment

Docker assets included:
- `services/api/Dockerfile`
- `apps/web/Dockerfile`
- `apps/web/nginx.conf`
- `docker-compose.yml`

Run full stack with Docker:

```bash
cd /home/escanor/donda
docker compose up --build
```

Services:
- Web: `http://127.0.0.1:5173`
- API: `http://127.0.0.1:8080`

## Developer Commands

Root `Makefile` shortcuts:
- `make test`
- `make run-api`
- `make run-web`
- `make docker-up`

Runtime controls:
- `API_KEY`: if set, requires `x-api-key` header on all API requests
- `RATE_LIMIT_PER_MIN`: request cap per key (or anonymous) per minute
- `INDEX_DB_PATH`: SQLite path for persistent decode index (default `data/chainindex.db`)
- `POLKADOT_SUBSCAN_API_KEY`: optional Subscan API key sent as `X-API-Key` for Polkadot decode calls

## CI

GitHub Actions workflow:
- `.github/workflows/ci.yml`

Checks:
- `cargo test` for `core/chainmerge`
- `cargo test` for `services/api`
- `npm install && npm run build` for `apps/web`
