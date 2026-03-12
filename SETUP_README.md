# ChainMerge Setup (Non-Docker)

This setup guide is for running ChainMerge on another local machine without Docker.

## 1) Prerequisites

Install:
- Rust toolchain (cargo + rustc)
- Node.js 18+ and npm
- Git

Quick checks:

```bash
rustc --version
cargo --version
node --version
npm --version
git --version
```

## 2) Clone Repository

```bash
git clone https://github.com/I-lost-everytime/Chaincodec.git
cd Chaincodec
```

## 3) Configure Environment

Copy `.env.example` to `.env`:

```bash
cp .env.example .env
```

Update `.env` values as needed.

Important for Polkadot:
- set `POLKADOT_SUBSCAN_API_KEY` in `.env`.

## 4) Install Web Dependencies

```bash
cd apps/web
npm install
cd ../..
```

## 5) Run Backend + Frontend

Use two terminals from repo root.

Terminal A (backend):

```bash
make make-api
```

Terminal B (frontend):

```bash
make run-web
```

Open:
- Web UI: `http://127.0.0.1:5173`
- API health: `http://127.0.0.1:8080/api/health`

## 6) Verify API Quickly

```bash
curl "http://127.0.0.1:8080/api/health"
curl "http://127.0.0.1:8080/api/examples"
```

Decode example (no rpc_url needed; backend has defaults):

```bash
curl "http://127.0.0.1:8080/api/decode?chain=ethereum&hash=0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad"
```

## 7) Common Commands

```bash
make test
make test-core
make test-api
make build-web
```

## 8) Troubleshooting

- `unsupported_event`:
  - hash is valid but not a transfer-like transaction for current decoder scope.
- `invalid_transaction_hash`:
  - wrong hash format or hash not found on the target chain/provider.
- Polkadot decode issues:
  - verify `POLKADOT_SUBSCAN_API_KEY` is set in `.env` and restart backend.

## 9) Notes

- This repository setup intentionally excludes Docker artifacts for now.
- RPC defaults and chain-specific overrides are managed in backend env config.
