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
  baseUrl: "http://127.0.0.1:8080", // or your hosted URL
  // apiKey: "optional-api-key",
});

async function main() {
  const tx = await client.decodeTx({
    chain: "ethereum",
    hash: "0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad",
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

- **`baseUrl`** (string, required): Base URL of the ChainMerge API (`http://127.0.0.1:8080`, `https://api.chainmerge.io`, etc.). No trailing slash.
- **`apiKey`** (string, optional): Sent as `x-api-key` header if provided.
- **`fetchImpl`** (function, optional): Custom `fetch` implementation for environments where `fetch` is not global (e.g. Node.js < 18).

### `client.decodeTx({ chain, hash, rpcUrl? })`

Decode a single transaction.

- **`chain`**: One of `"solana" | "ethereum" | "cosmos" | "aptos" | "sui" | "polkadot" | "bitcoin" | "starknet"`.
- **`hash`**: Transaction hash string.
- **`rpcUrl`** (optional): Override RPC URL for this request; if omitted, ChainMerge's internal defaults and env config are used.

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
