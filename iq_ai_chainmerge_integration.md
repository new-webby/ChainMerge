---
title: IQ AI × ChainMerge Integration Guide
description: How to build multichain, autonomous DeFAI agents using ChainMerge’s normalized blockchain data and IQ AI’s on‑chain agent platform.
---

## 🧠 IQ AI × 🔗 ChainMerge

**Goal:** Build **autonomous, tokenized DeFAI agents** that can **reason over multichain activity** and **act on‑chain**, using:

- **IQ AI** → DeFAI‑focused platform + TypeScript Agent Development Kit for on‑chain agents.
- **ChainMerge** → Multichain infrastructure + SDK that turns raw blockchain transactions into a single, deterministic JSON schema.

Together they enable **“signal‑backed” agents**: IQ AI agents that consume ChainMerge’s normalized events and execute cross‑chain strategies.

---

## 1. Project Overviews

### 1.1 IQ AI

- **Type:** DeFAI‑focused platform for **autonomous, tokenized AI agents on‑chain**.
- **Dev Experience:** Open‑source **TypeScript Agent Development Kit (ADK)**.
- **What you can build:**
  - Simple automated bots.
  - Autonomous on‑chain AI agents.
  - Complex **multi‑agent systems** coordinating strategies.
- **Primary value:** Encodes on‑chain logic, risk rules, and DeFi strategies into agents that can own assets and act without constant human input.

### 1.2 ChainMerge

- **Type:** **Multichain infrastructure** and SDK that normalizes raw blockchain transaction data.
- **Problem it solves:** Every chain has different encodings (EVM logs, Solana instructions, Cosmos Protobufs, Bitcoin UTXO, etc.) making multichain apps painful to build.
- **Core components:**
  - **Universal Decoders** → deep parsers that output a unified `NormalizedTransaction` shape.
  - **Semantic Event Extraction** → identifies high‑level events like `token_transfer`, `nft_mint`, `swap`.
  - **ChainMerge SDK (`chainmerge-sdk`)** → ergonomic TypeScript/JavaScript client.
  - **Indexing & Health APIs** → `decodeAndIndexTx`, `lookupIndexedTx`, `listRecentIndexedTxs`, `health`, `metrics`, etc.
- **Supported chains (v1):**
  - `solana`, `ethereum`, `cosmos`, `aptos`, `sui`, `polkadot`, `bitcoin`, `starknet`.

---

## 2. Why IQ AI + ChainMerge fit together

### 2.1 Roles in the stack

- **ChainMerge = Signal Layer**
  - Normalizes raw blockchain data into deterministic, LLM‑friendly JSON.
  - Classifies user‑level intent via semantic events (e.g., “large swap”, “bridge transfer”, “whale token transfer”).

- **IQ AI = Agent / Action Layer**
  - Hosts tokenized agents that **observe signals**, **decide**, and **act on‑chain**.
  - Encapsulates DeFi strategies, risk limits, and execution behavior.

### 2.2 Benefits of this pairing

- **Multichain by default:** Agents can react to activity across 8+ ecosystems using a **single schema**.
- **Fewer hallucinations:** Agents or upstream LLMs reason over clean, deterministic JSON instead of raw hex or logs.
- **Composable strategies:** Different IQ AI agents can subscribe to different semantic patterns from the same ChainMerge backend.
- **Faster iteration:** You don’t build or maintain chain‑specific indexers; you focus on strategy + UX.

---

## 3. Core Data Model: ChainMerge types

ChainMerge exposes a stable, typed interface via `chainmerge-sdk`:

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

These types are exported directly from the SDK:

```ts
import type { NormalizedTransaction, NormalizedEvent, Chain } from "chainmerge-sdk";
```

This is the **primary payload** you feed into IQ AI agents for analysis and strategy decisions.

---

## 4. Best‑case integration architecture

### 4.1 High‑level flow

1. **Ingest & normalize**
   - A backend service uses `chainmerge-sdk` + the ChainMerge API to:
     - Decode transactions (`decodeTx` / `decodeAndIndexTx`).
     - Maintain an index of interesting transactions (via `listRecentIndexedTxs` etc.).

2. **Dispatch to agents**
   - The same backend (or a message bus / pub‑sub layer) forwards `NormalizedTransaction` objects to IQ AI agents.
   - Each agent subscribes to a subset of chains, addresses, or semantic event patterns.

3. **Decide & act**
   - IQ AI agents:
     - Evaluate incoming normalized data.
     - Optionally call out to LLMs for higher‑level reasoning using the deterministic JSON.
     - Execute on‑chain actions (swaps, rebalances, hedges, alerts) according to encoded strategies.

4. **Observe & iterate**
   - ChainMerge continues to index new activity.
   - Agents continuously refine behavior based on performance metrics and updated signals.

### 4.2 Visual mental model

- **Bottom:** Raw chains (Ethereum, Solana, Cosmos, etc.).
- **Middle (ChainMerge):** Normalization + semantic event extraction + indexing.
- **Top (IQ AI):** Strategy engines and multi‑agent systems that own wallets/contracts and act on insights.

---

## 5. Concrete integration using `chainmerge-sdk`

### 5.1 Create a ChainMerge client in your agent runtime

```ts
import { ChainMergeClient } from "chainmerge-sdk";

const chainmerge = new ChainMergeClient({
  baseUrl: "https://api.yourdomain.com/chainmerge",
  apiKey: process.env.CHAINMERGE_API_KEY, // optional
});
```

### 5.2 Fetch & normalize a transaction before passing to IQ AI

This is a minimal example of how a service can decode a transaction and forward it to an IQ AI agent.

```ts
import type { NormalizedTransaction } from "chainmerge-sdk";
// import { createIqAgent } from "@iqai/adk"; // hypothetical IQ AI ADK import

async function handleTx(chain: "ethereum" | "solana", hash: string) {
  const normalized: NormalizedTransaction = await chainmerge.decodeTx({
    chain,
    hash,
  });

  // Example: forward to an IQ AI agent (pseudocode)
  await iqAgent.handleNormalizedTransaction(normalized);
}
```

In a real system, `handleTx` would be triggered by:

- Webhooks from your RPC/indexer.
- A polling loop over `listRecentIndexedTxs`.
- A queue that your IQ AI agents listen to.

---

## 6. Example: “Signal‑backed arbitrage agent”

### 6.1 Concept

Build an IQ AI agent that:

- Watches **large swaps** on Ethereum & Solana (via ChainMerge).
- Detects mispricing opportunities across DEXs or chains.
- Executes predefined arbitrage or hedging strategies on‑chain.

### 6.2 Pseudocode sketch

```ts
// Pseudocode: an IQ AI strategy consuming ChainMerge data

async function onNormalizedTransaction(tx: NormalizedTransaction) {
  for (const ev of tx.events) {
    if (ev.event_type !== "token_transfer") continue;

    const size = BigInt(ev.amount ?? "0");
    const isWhale = size > 1_000_000n * 10n ** 6n; // example USDC threshold

    if (!isWhale) continue;

    // Ask an IQ AI strategy / LLM to evaluate the opportunity (optional)
    const decision = await strategyEngine.evaluate({
      chain: tx.chain,
      txHash: tx.tx_hash,
      token: ev.token,
      from: ev.from,
      to: ev.to,
      amount: ev.amount,
    });

    if (decision.type === "ARBITRAGE_OPPORTUNITY") {
      await iqAgent.executeArb(decision.plan);
    }
  }
}
```

Here, **ChainMerge** provides clean, structured context; **IQ AI** owns the execution and risk logic.

---

## 7. Multi‑agent patterns

Using IQ AI’s support for multi‑agent systems, you can compose several specialized agents, all reading from the same ChainMerge signal layer:

- **Watcher agent**
  - Subscribes to ChainMerge events.
  - Tags and routes `NormalizedTransaction`s to other agents (e.g., arbitrage, NFT, risk).

- **Arbitrage agent**
  - Focuses on DEX/bridge pricing mismatches.

- **Risk / Compliance agent**
  - Monitors for blacklisted addresses, unusually large flows, or anomalous behavior.

- **Narrator / Analytics agent**
  - Converts normalized events into human‑readable summaries or dashboards.

Each agent:

- Receives `NormalizedTransaction` payloads from ChainMerge.
- Applies its own policy and state.
- Optionally sends transactions back to the chain or messages to other agents.

---

## 8. Deployment & environment notes

- **Backend (ChainMerge API)**
  - Configure:
    - `HOST`, `PORT`
    - `CHAINMERGE_RPC_URL` / `CHAINMERGE_RPC_URL_<CHAIN>`
    - `API_KEY` (if you require authenticated access)
    - `INDEX_DB_PATH` for persisted indexing.

- **Agent runtime (IQ AI)**
  - Run in the same private network/VPC as your ChainMerge API for low‑latency reads.
  - Keep secrets like `API_KEY` server‑side; browser clients should call through your backend.

---

## 9. Summary

- **ChainMerge** gives you a **single, deterministic JSON format** for multichain transactions plus semantic event extraction.
- **IQ AI** turns that signal into **autonomous, tokenized agents** that can hold assets, execute strategies, and coordinate as multi‑agent systems.
- The **best‑case integration** is:
  - ChainMerge = **multichain signal + normalization layer**.
  - IQ AI = **on‑chain strategy + execution layer**.
- This combination unlocks the next generation of **signal‑backed DeFAI applications**: arbitrageurs, storytellers, cross‑chain wallets, and more—without rebuilding chain‑specific plumbing.

