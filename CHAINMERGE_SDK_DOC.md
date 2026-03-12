# ChainMerge SDK Doc (`chainmerge-sdk`)

This document explains **end‑to‑end usage** of the ChainMerge JavaScript/TypeScript SDK:

- What the SDK does
- How to install and configure it
- How to call the API from Node, browsers, and frameworks
- How to handle errors, timeouts, and environment configuration

---

### 1. What the SDK does

The SDK wraps the ChainMerge HTTP API and gives you a typed client:

- **Input**: `chain`, transaction `hash`, optional `rpcUrl`
- **Output**: a **normalized transaction** object with a single JSON shape across chains.

Supported chains:

- `solana`, `ethereum`, `cosmos`, `aptos`, `sui`, `polkadot`, `bitcoin`, `starknet`

You do **not** need to write custom decoders per chain. Your app just consumes one schema.

---

### 2. Installation

Install from npm:

```bash
npm install chainmerge-sdk
# or
yarn add chainmerge-sdk
```

The package ships TypeScript types and works in:

- Node.js (18+ recommended, or with a custom `fetch` polyfill)
- Browser apps (pure JS, React, Next.js, etc.)

---

### 3. Core concepts and types

The SDK mirrors the shared core types and API response schema.

#### 3.1 Chain keys

```ts
type Chain =
  | "solana"
  | "ethereum"
  | "cosmos"
  | "aptos"
  | "sui"
  | "polkadot"
  | "bitcoin"
  | "starknet";
```

#### 3.2 Normalized transaction

```ts
interface NormalizedEvent {
  event_type: "token_transfer" | "unsupported";
  token?: string;
  from?: string;
  to?: string;
  amount?: string;
  raw_program?: string;
}

interface NormalizedTransaction {
  chain: Chain;
  tx_hash: string;
  sender?: string;
  receiver?: string;
  value?: string;
  events: NormalizedEvent[];
}
```

The exact TypeScript definitions are exported from the package, so you can import them directly:

```ts
import type { NormalizedTransaction, NormalizedEvent, Chain } from "chainmerge-sdk";
```

---

### 4. Creating a client

The main entry is `ChainMergeClient`.

```ts
import { ChainMergeClient } from "chainmerge-sdk";

const client = new ChainMergeClient({
  baseUrl: "http://127.0.0.1:8080", // or your hosted API URL
  apiKey: process.env.CHAINMERGE_API_KEY, // optional
});
```

#### 4.1 Constructor options

- **`baseUrl`** (string, required)
  - Base URL of your ChainMerge API.
  - Examples:
    - Local: `http://127.0.0.1:8080`
    - Hosted: `https://api.yourdomain.com/chainmerge`
  - Do **not** include a trailing slash.

- **`apiKey`** (string, optional)
  - If your API is configured with `API_KEY`, every request must include `x-api-key`.
  - The SDK sends this header automatically when `apiKey` is provided.

- **`fetchImpl`** (function, optional)
  - Custom `fetch` implementation.
  - Use when running on Node < 18:

    ```ts
    import fetch from "node-fetch";
    import { ChainMergeClient } from "chainmerge-sdk";

    const client = new ChainMergeClient({
      baseUrl: "http://127.0.0.1:8080",
      fetchImpl: fetch as unknown as typeof globalThis.fetch,
    });
    ```

---

### 5. Decoding a transaction

The primary method is `decodeTx`.

```ts
const tx = await client.decodeTx({
  chain: "ethereum",
  hash: "0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad",
});
```

#### 5.1 Parameters

- **`chain`**: one of the supported `Chain` keys.
- **`hash`**: transaction hash/digest string.
- **`rpcUrl`** (optional): per‑request RPC URL override; most apps can omit this and rely on backend defaults.

#### 5.2 Return value

- Resolves to a `NormalizedTransaction`.
- Throws on error (see **Error handling** below).

#### 5.3 Minimal Node example

```ts
import { ChainMergeClient } from "chainmerge-sdk";

async function main() {
  const client = new ChainMergeClient({
    baseUrl: "http://127.0.0.1:8080",
  });

  const tx = await client.decodeTx({
    chain: "ethereum",
    hash: "0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad",
  });

  console.log("normalized tx:", tx);

  for (const ev of tx.events) {
    if (ev.event_type === "token_transfer") {
      console.log(
        `Token transfer of ${ev.amount} from ${ev.from} to ${ev.to} (token: ${ev.token})`,
      );
    }
  }
}

main().catch((err) => {
  console.error("decode failed:", err);
});
```

---

### 6. Browser / React usage

You can use the SDK directly from browser apps, including React and similar frameworks, as long as your API has CORS enabled (the API already configures permissive CORS).

#### 6.1 Simple React hook example

```ts
import { useState } from "react";
import { ChainMergeClient, type NormalizedTransaction } from "chainmerge-sdk";

const client = new ChainMergeClient({
  baseUrl: "http://127.0.0.1:8080",
});

export function UseDecodeExample() {
  const [chain, setChain] = useState<"ethereum" | "solana">("ethereum");
  const [hash, setHash] = useState("");
  const [tx, setTx] = useState<NormalizedTransaction | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function onDecode() {
    setLoading(true);
    setError(null);
    setTx(null);

    try {
      const result = await client.decodeTx({ chain, hash });
      setTx(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div>
      <select value={chain} onChange={(e) => setChain(e.target.value as "ethereum" | "solana")}>
        <option value="ethereum">ethereum</option>
        <option value="solana">solana</option>
      </select>

      <input
        value={hash}
        onChange={(e) => setHash(e.target.value)}
        placeholder="transaction hash"
      />

      <button onClick={onDecode} disabled={loading}>
        {loading ? "Decoding..." : "Decode"}
      </button>

      {error && <pre style={{ color: "red" }}>{error}</pre>}
      {tx && <pre>{JSON.stringify(tx, null, 2)}</pre>}
    </div>
  );
}
```

---

### 7. Error handling

On non‑2xx responses from the API, the SDK throws an `Error` with extra metadata:

- `error.message`: human‑readable explanation
- `error.code`: backend error code (e.g. `invalid_request`, `invalid_transaction_hash`, `unsupported_event`)
- `error.retryable`: boolean indicating whether retry is likely to succeed

Example:

```ts
try {
  const tx = await client.decodeTx({
    chain: "ethereum",
    hash: "not-a-valid-hash",
  });
} catch (err) {
  if (err instanceof Error) {
    console.error("decode failed:", err.message);
    const code = (err as any).code;
    const retryable = (err as any).retryable;
    console.error("code:", code, "retryable:", retryable);
  }
}
```

You can use `code` and `retryable` to map into application‑specific error messages or retry logic.

---

### 8. Environment and deployment considerations

#### 8.1 Backend environment variables

The SDK itself is stateless; it just calls your API. Your **API** can be configured via:

- `HOST`, `PORT`
- `CHAINMERGE_RPC_URL` / `CHAINMERGE_RPC_URL_<CHAIN>`
- `API_KEY` (enable required `x-api-key`)
- `RATE_LIMIT_PER_MIN`
- `INDEX_DB_PATH`
- `POLKADOT_SUBSCAN_API_KEY`

Ensure these are correctly set in your deployment (Docker, Kubernetes, server VM, etc.).

#### 8.2 Frontend environments

In browser apps, keep secrets out of the client:

- If you use `API_KEY`, terminate requests on a backend or edge function that adds the header and call that from the browser.
- Alternatively, leave `API_KEY` unset for local development and internal tools.

---

### 9. Typical integration patterns

- **Wallets**
  - Use `decodeTx` to show human‑readable transaction summaries across chains.
  - Use `events` to label transfers and token movements.

- **Dapps / backends**
  - Store `NormalizedTransaction` in your database keyed by `chain` + `tx_hash`.
  - Power notifications, analytics, or dashboards from one schema.

- **Indexers / monitoring**
  - Use `/api/index/*` endpoints plus the SDK (or raw HTTP) to build higher‑level indexing or alerting systems.

---

### 10. Troubleshooting

- **`TypeError: fetch is not a function` in Node**
  - Use Node 18+ or pass a `fetchImpl` (e.g. from `node-fetch`).

- **CORS errors in the browser**
  - Ensure your API is reachable and CORS is enabled. The default API uses a permissive CORS layer (`Any` origin), but confirm your deployment matches.

- **`invalid_transaction_hash`**
  - Check that the hash format matches the chain (length, prefix, etc.).

- **`unsupported_event`**
  - The transaction was fetched and parsed, but it doesn’t match a transfer‑like pattern the current decoders understand.

If you need examples tailored to a specific framework (Next.js, Express middleware, etc.), you can base them directly on the patterns in this document. 
