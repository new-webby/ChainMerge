# ChainMerge Runbook

## Local Development

Fast path with Make:

```bash
cd /home/escanor/donda
make dev
```

Manual path:

1. Start API:

```bash
cd /home/escanor/donda/services/api
cargo run
```

2. Start web app:

```bash
cd /home/escanor/donda/apps/web
npm install
npm run dev
```

3. Open UI:
- `http://127.0.0.1:5173`

## Useful API Endpoints

- Health: `GET /api/health`
- Examples: `GET /api/examples`
- Metrics: `GET /api/metrics`
- Decode: `GET /api/decode?chain=<chain>&hash=<tx>&rpc_url=<url>`
- Decode+Index: `GET /api/index/decode?chain=<chain>&hash=<tx>&rpc_url=<url>`
- Index lookup: `GET /api/index/{chain}/{hash}`
- Index recent: `GET /api/index/recent?limit=20`

## Chain RPC Expectations

- `ethereum`: JSON-RPC endpoint (Alchemy/Infura/etc.)
- `solana`: Solana JSON-RPC endpoint
- `cosmos`: Cosmos LCD/REST base URL
- `aptos`: Aptos fullnode REST base URL
- `sui`: Sui JSON-RPC endpoint
- `bitcoin`: Blockstream-compatible REST base URL (e.g. `/tx/{hash}`)
- `polkadot`: Subscan-compatible API base URL (for extrinsic lookup)
- `starknet`: Starknet JSON-RPC endpoint

Weighted failover format supported in `rpc_url`:
- `<url-a>|3,<url-b>|1`
- Higher weight means higher selection priority; automatic failover and circuit-open backoff apply.

## Runtime Controls

- `API_KEY`: when set, all requests require header `x-api-key: <API_KEY>`
- `RATE_LIMIT_PER_MIN`: max requests per key (or anonymous) per minute
- `INDEX_DB_PATH`: SQLite file path for persistent index storage
- `POLKADOT_SUBSCAN_API_KEY`: optional Subscan API key used as `X-API-Key` for Polkadot extrinsic lookups

## Docker Deployment

```bash
cd /home/escanor/donda
make docker-up
```

- Web: `http://127.0.0.1:5173`
- API: `http://127.0.0.1:8080`

## Common Make Targets

- `make test`
- `make test-core`
- `make test-api`
- `make build-web`
- `make run-api`
- `make run-web`
- `make docker-up`
- `make docker-down`
