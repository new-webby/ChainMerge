# Startup Guide

This guide is focused on a reliable local startup flow.

## Prerequisites

- Rust toolchain (`cargo`)
- Node.js + npm
- Docker (optional, for container run)

## Local Startup (Recommended)

Open two terminals.

From repo root in both:

```bash
cd /home/escanor/donda
```

Terminal 1 (API):

```bash
make run-api
```

Terminal 2 (Web):

```bash
make run-web
```

Open:
- Web UI: `http://127.0.0.1:5173`
- API health: `http://127.0.0.1:8080/api/health`

## Manual Startup (Without Make)

Terminal 1 (API):

```bash
cd /home/escanor/donda/services/api
cargo run
```

Terminal 2 (Web):

```bash
cd /home/escanor/donda/apps/web
npm install
npm run dev
```

## API Runtime Configuration

Common API env vars:
- `HOST` (default `0.0.0.0`)
- `PORT` (default `8080`)
- `CHAINMERGE_RPC_URL` optional global RPC override when query `rpc_url` is missing
- `CHAINMERGE_RPC_URL_ETHEREUM` optional chain-specific override
- `CHAINMERGE_RPC_URL_SOLANA` optional chain-specific override
- `CHAINMERGE_RPC_URL_COSMOS` optional chain-specific override
- `CHAINMERGE_RPC_URL_APTOS` optional chain-specific override
- `CHAINMERGE_RPC_URL_SUI` optional chain-specific override
- `CHAINMERGE_RPC_URL_POLKADOT` optional chain-specific override
- `CHAINMERGE_RPC_URL_BITCOIN` optional chain-specific override
- `CHAINMERGE_RPC_URL_STARKNET` optional chain-specific override
- `API_KEY` enables `x-api-key` requirement
- `RATE_LIMIT_PER_MIN` request cap per minute
- `INDEX_DB_PATH` SQLite index path
- `POLKADOT_SUBSCAN_API_KEY` Subscan API key for Polkadot decode

Example API startup with env vars:

```bash
cd /home/escanor/donda/services/api
export HOST=0.0.0.0
export PORT=8080
export RATE_LIMIT_PER_MIN=240
export INDEX_DB_PATH=data/chainindex.db
cargo run
```

## Startup Verification

Health:

```bash
curl "http://127.0.0.1:8080/api/health"
```

Examples:

```bash
curl "http://127.0.0.1:8080/api/examples"
```

Decode smoke test:

```bash
curl "http://127.0.0.1:8080/api/decode?chain=ethereum&hash=0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad"
```

## Docker Startup

```bash
cd /home/escanor/donda
export POLKADOT_SUBSCAN_API_KEY="<your_subscan_key>"
make docker-up
```

Services:
- Web: `http://127.0.0.1:5173`
- API: `http://127.0.0.1:8080`

## Shutdown

If started with `make run-*` or manual commands, stop each process with `Ctrl+C`.

If started with Docker:

```bash
cd /home/escanor/donda
make docker-down
```

## Troubleshooting

- `invalid_request` for Polkadot endpoint
  - use Subscan base URL (`https://polkadot.api.subscan.io`) and API key
- port already in use on `8080` or `5173`
  - stop previous processes, then restart
