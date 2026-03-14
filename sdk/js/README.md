# chainmerge-sdk

TypeScript/JavaScript SDK for the ChainMerge multichain transaction decoder API.

ChainMerge normalizes transactions from multiple chains (Ethereum, Solana, Cosmos, Aptos, Sui, Polkadot, Bitcoin, Starknet, etc.) into a single JSON shape. This SDK makes it easy for dapps, wallets, and backends to call the `/api/decode` endpoint and work with typed results.

## Installation

```bash
npm install chainmerge-sdk
# or
yarn add chainmerge-sdk
```

## Quick start

```ts
import { ChainMergeClient } from "chainmerge-sdk";

const client = new ChainMergeClient({
  // baseUrl is optional, defaults to "https://api.chainmerge.io"
  // apiKey: "optional-api-key",
});

async function main() {
  const tx = await client.decodeTx({
    chain: "ethereum",
    hash: "0x8999.......",
  });

  console.log("normalized tx:", tx);

  for (const event of tx.events) {
    if (event.event_type === "token_transfer") {
      console.log(
        `Token transfer of ${event.amount} from ${event.from} to ${event.to} (token: ${event.token})`,
      );
    }
  }
}

main().catch((err) => {
  console.error("decode failed:", err);
});
```

## API

### `new ChainMergeClient(options)`

- **`baseUrl`** (string, optional): Base URL of the ChainMerge API. Defaults to `https://api.chainmerge.io`. No trailing slash.
- **`apiKey`** (string, optional): Sent as `x-api-key` header if provided.
- **`fetchImpl`** (function, optional): Custom `fetch` implementation for environments where `fetch` is not global (e.g. Node.js < 18).

### `client.decodeTx({ chain, hash, rpcUrl? })`

Decode a single transaction.

- **`chain`**: One of `"solana" | "ethereum" | "cosmos" | "aptos" | "sui" | "polkadot" | "bitcoin" | "starknet"`.
- **`hash`**: Transaction hash string.
- **`rpcUrl`** (optional): Override RPC URL for this request.

### `client.health()`
Returns a `Promise<HealthResponse>` with `{ status: 'ok', service: 'chainmerge-api' }`.

### `client.examples()`
Returns a `Promise<ExamplesResponse>` containing supported example transactions and chains.

### `client.metrics()`
Returns a `Promise<any>` with internal API metrics.

### `client.decodeAndIndexTx({ chain, hash, rpcUrl? })`
Decodes and stores the transaction in the index database.

### `client.lookupIndexedTx({ chain, hash })`
Retrieves a previously indexed transaction.

### `client.listRecentIndexedTxs({ limit? })`
Lists recently indexed transactions.

Returns a `Promise<NormalizedTransaction>`, where:

- `NormalizedTransaction` includes:
  - `chain`, `tx_hash`, `sender?`, `receiver?`, `value?`
  - `events: NormalizedEvent[]`, with event types like `"token_transfer"`.

Errors from the API are thrown as `Error` instances with extra properties:

- `error.code`
- `error.retryable`

## Publishing (repo maintainers)

From the `sdk/js` directory:

```bash
npm install
npm run build

# first time: npm login
npm publish --access public
```

Make sure you bump the `version` field in `package.json` before publishing a new release.
