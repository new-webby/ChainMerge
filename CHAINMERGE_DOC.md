# ChainMerge Doc

### Overview

**ChainMerge** is a multichain transaction decoder. You give it a `chain` and a transaction `hash`, and it returns one **normalized JSON schema** across supported chains (`solana`, `ethereum`, `cosmos`, `aptos`, `sui`, `polkadot`, `bitcoin`, `starknet`).

Input:
- `chain`
- `hash` (transaction hash/digest)
- `rpc_url` (optional override; backend has built-in per‑chain defaults)

Output:
- one normalized JSON response schema across supported chains.

### Running ChainMerge locally

- Follow `SETUP_README.md` to install Rust, Node, configure `.env`, and install web dependencies.
- From repo root:
  - **Backend**: `make make-api` (or `make run-api` depending on your Makefile)
  - **Frontend**: `make run-web`
- Default ports:
  - Web UI: `http://127.0.0.1:5173`
  - API: `http://127.0.0.1:8080`

### Supported chains and RPC expectations

- **Supported chain keys**:
  - `solana`, `ethereum`, `cosmos`, `aptos`, `sui`, `polkadot`, `bitcoin`, `starknet`

- **RPC URL expectations**:
  - `ethereum`: EVM JSON‑RPC endpoint
  - `solana`: Solana JSON‑RPC endpoint
  - `cosmos`: Cosmos REST/LCD base URL
  - `aptos`: Aptos fullnode REST base URL
  - `sui`: Sui JSON‑RPC endpoint
  - `bitcoin`: Blockstream‑compatible REST base URL
  - `polkadot`: Subscan API base URL (`https://polkadot.api.subscan.io`)
  - `starknet`: Starknet JSON‑RPC endpoint

Note:
- Polkadot decode currently uses Subscan API shape, not node JSON‑RPC.
- Subscan typically requires `POLKADOT_SUBSCAN_API_KEY`.

### Using the web app

1. Start API and web app.
2. Open `http://127.0.0.1:5173`.
3. Select a chain.
4. Enter a transaction hash.
5. Optionally set an RPC URL override.
6. Click **Decode Transaction** to see normalized JSON.

### Using the HTTP API directly

**Health:**

```bash
curl "http://127.0.0.1:8080/api/health"
```

**Examples:**

```bash
curl "http://127.0.0.1:8080/api/examples"
```

**Decode:**

```bash
curl "http://127.0.0.1:8080/api/decode?chain=ethereum&hash=0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad"
```

**Decode + index:**

```bash
curl "http://127.0.0.1:8080/api/index/decode?chain=ethereum&hash=0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad"
```

**Lookup indexed tx:**

```bash
curl "http://127.0.0.1:8080/api/index/ethereum/0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad"
```

**Recent indexed txs:**

```bash
curl "http://127.0.0.1:8080/api/index/recent?limit=20"
```

### Using the JavaScript/TypeScript SDK

For dapps, wallets, or backend services, use the published npm package instead of calling the HTTP API manually.

**Install:**

```bash
npm install chainmerge-sdk
# or
yarn add chainmerge-sdk
```

**Example (Node / browser):**

```ts
import { ChainMergeClient } from "chainmerge-sdk";

const client = new ChainMergeClient({
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

### Runtime configuration

Key environment variables:

- `HOST`: API bind host (default `0.0.0.0`)
- `PORT`: API port (default `8080`)
- `CHAINMERGE_RPC_URL`: optional global RPC URL override when query `rpc_url` is omitted
- `CHAINMERGE_RPC_URL_<CHAIN>`: chain‑specific overrides, e.g.:
  - `CHAINMERGE_RPC_URL_ETHEREUM`
  - `CHAINMERGE_RPC_URL_SOLANA`
  - `CHAINMERGE_RPC_URL_COSMOS`
  - `CHAINMERGE_RPC_URL_APTOS`
  - `CHAINMERGE_RPC_URL_SUI`
  - `CHAINMERGE_RPC_URL_POLKADOT`
  - `CHAINMERGE_RPC_URL_BITCOIN`
  - `CHAINMERGE_RPC_URL_STARKNET`
- `API_KEY`: if set, requests must send `x-api-key`
- `RATE_LIMIT_PER_MIN`: request limit per minute per key/client
- `INDEX_DB_PATH`: SQLite path for indexed decode storage
- `POLKADOT_SUBSCAN_API_KEY`: sent as `X-API-Key` to Subscan

### Common errors

- `invalid_request: polkadot decoder expects a Subscan API base URL`
  - Use `https://polkadot.api.subscan.io`, not `https://rpc.polkadot.io`.
- `invalid_request: Subscan API key required`
  - Set `POLKADOT_SUBSCAN_API_KEY`.
- `invalid_transaction_hash`
  - Hash format does not match chain rules for the given chain.
- `unsupported_event`
  - Transaction parsed, but no transfer‑like event matched current decoder logic.

