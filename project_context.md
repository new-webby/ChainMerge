ChainMerge & ChainKit
A Unified Multichain Data Decoding and Infrastructure Layer

1. Introduction
The blockchain ecosystem has rapidly evolved into a multi-chain world. Developers no longer build applications on a single blockchain. Instead, modern decentralized applications interact with multiple chains such as Ethereum, Solana, Cosmos, Polkadot, Aptos, Sui, Bitcoin, and StarkNet.
However, each blockchain ecosystem has its own data formats, encoding systems, transaction structures, and event models. This creates a major challenge for developers building multichain infrastructure, analytics tools, wallets, and DeFi applications.
Every time a new chain is added to a project, developers must rebuild the entire data decoding and parsing layer, increasing development complexity and maintenance cost.
This project aims to solve this fragmentation by introducing ChainMerge and ChainKit, a unified foundation layer for decoding and interacting with multiple blockchain networks.

2. Problem Statement
2.1 Fragmented Blockchain Data Encoding
Different blockchains use completely different serialization and encoding formats.
Blockchain
Encoding Format
Ethereum
ABI Encoding
Solana
Borsh
Cosmos
Protobuf
Aptos / Sui
BCS
Polkadot
SCALE
Bitcoin
UTXO transaction model
StarkNet
Cairo ABI

Although all blockchains fundamentally represent transactions, accounts, and events, the decoding rules and structures vary significantly.
As a result, developers building multichain applications face several problems:
1. Rewriting Parsing Layers
Every chain requires a custom decoding implementation. A project supporting five chains may maintain five completely different parsing systems.
2. Increased Engineering Complexity
Different serialization formats require different libraries, SDKs, and error-handling mechanisms.
3. Inconsistent Data Formats
Even when performing the same operation (for example token transfers), the output structures differ across chains.
4. Difficult Multichain Expansion
Adding support for a new chain often means rebuilding large portions of the backend infrastructure.
5. Developer Productivity Loss
Developers spend more time writing decoding logic than actually building application features.

3. Target Users
The project primarily targets developers and infrastructure builders working in the blockchain ecosystem.
Target groups include:
• DeFi protocol developers
• Blockchain analytics platforms
• Cross-chain dashboards
• Wallet developers
• Blockchain indexers
• Data engineering teams
• AI agents interacting with blockchain data
• Multichain infrastructure providers
These developers require normalized blockchain data before they can build useful applications.

4. Proposed Solution
To address this fragmentation problem, this project proposes a unified blockchain decoding and infrastructure layer composed of two parts:
ChainMerge (Core Decoder Layer)
ChainKit (Infrastructure Toolkit)
Together they create a standardized way to interact with blockchain transaction data across multiple networks.

5. Project Vision (Full Version)
ChainMerge + ChainKit (Full Infrastructure)
The complete project aims to become a foundation layer for multichain development, simplifying blockchain data access and decoding.
ChainMerge
ChainMerge acts as a universal blockchain decoder capable of interpreting transactions from multiple chains and converting them into a standardized data format.
Instead of writing separate parsers for each chain, developers can simply use ChainMerge.
Example:
Input:
Raw blockchain transaction

Output:
Normalized structured data

Example Output Format
{
 chain: "ethereum",
 tx_hash: "...",
 sender: "...",
 receiver: "...",
 value: "...",
 events: [...]
}

The same format would be generated regardless of the source chain.

ChainKit Components
The full infrastructure also includes several supporting modules:
1. chainerrors
Standardized decoding of blockchain errors.
Examples include:
• Ethereum revert messages
• Solana program errors
• Cosmos module errors
These are normalized into a single error format.

2. chainrpc
A reliable RPC communication layer that provides:
• RPC failover
• node redundancy
• automatic retries
• load balancing between nodes

3. chainindex
A lightweight indexing engine capable of:
• streaming blockchain events
• handling chain reorganizations
• building searchable transaction databases

Project Vision
The long-term goal is to build a universal foundation layer for multichain infrastructure where developers can interact with any blockchain using a consistent API and data structure.

6. Hackathon Implementation (ChainMerge Lite)
Since the full project scope is large, a simplified version called ChainMerge Lite will be developed for the hackathon.
This version focuses only on the core decoding problem.

ChainMerge Lite Overview
ChainMerge Lite will be a universal transaction decoder supporting a limited number of blockchains.
Supported Chains
• Ethereum
• Solana
• Cosmos (optional depending on time)

System Workflow
User provides a transaction hash and selects a blockchain network.
The system fetches the raw transaction data from the chain.
The decoder interprets the transaction using the appropriate decoding logic.
The output is converted into a standardized JSON format.

Example Interface
User Input:
Transaction Hash
Blockchain Network

System Output:
Normalized Transaction Data


Example Standardized Output
{
 chain: "solana",
 tx_hash: "...",
 sender: "...",
 receiver: "...",
 value: "...",
 events: [
   {
     type: "token_transfer",
     token: "...",
     from: "...",
     to: "...",
     amount: "..."
   }
 ]
}


7. Key Innovation
One of the most powerful features of this project is event normalization across blockchains.
For example:
Different blockchains represent token transfers differently.
Chain
Event Name
Ethereum
Transfer
Solana
SPL Transfer
Cosmos
Coin Transfer

ChainMerge converts all of them into a single unified event format.
Example:
{
 type: "token_transfer",
 token: "...",
 from: "...",
 to: "...",
 amount: "..."
}

This allows applications to handle blockchain data without chain-specific logic.

8. Difference Between Full Vision and Hackathon Version
Feature
Full Project
Hackathon Version
Multichain Decoder
Yes
Yes
RPC Failover Layer
Yes
No
Blockchain Indexing
Yes
No
Error Standardization
Yes
Optional
Supported Chains
Many
2–3
Language Bindings
Rust, Python, npm, WASM
Rust + Node
Production Ready
Yes
Prototype

The hackathon implementation focuses on demonstrating the core concept rather than building the entire infrastructure.

9. Technology Stack
Core Engine
Rust
Rust is chosen because of its:
• high performance
• memory safety
• suitability for blockchain infrastructure

Decoding Libraries
Depending on the chain:
Ethereum → ABI decoding libraries
Solana → Borsh serialization libraries
Cosmos → Protobuf decoding tools

Backend
Node.js / Express
Used for building the API that interacts with ChainMerge.

Frontend
React.js
Provides a simple interface where users can:
• input transaction hashes
• select chains
• view decoded outputs

Additional Technologies
WebAssembly (WASM)
Allows the Rust core to run in web environments.
Docker (optional)
For easy deployment.

10. System Architecture
High Level Architecture
User Interface
      |
      v
API Layer
      |
      v
ChainMerge Core
      |
  ┌───┼──────────────┐
  |   |              |
EVM Decoder   Solana Decoder   Cosmos Decoder
  |   |              |
  └───┴──────────────┘
      |
Normalized Output


11. Expected Impact
If successfully developed, ChainMerge and ChainKit could significantly improve the developer experience in the blockchain ecosystem.
Benefits include:
• faster multichain development
• reduced infrastructure complexity
• standardized blockchain data
• easier integration of new chains
Developers could build multichain applications without rewriting their parsing logic every time a new blockchain is added.

12. Conclusion
The rapid expansion of the blockchain ecosystem has created a significant challenge for developers building multichain applications.
ChainMerge and ChainKit aim to address this challenge by creating a unified decoding and infrastructure layer that standardizes blockchain transaction data across multiple networks.
While the full vision includes RPC layers, indexing systems, and standardized error decoding, the hackathon implementation focuses on demonstrating the core concept of universal transaction decoding.
By simplifying how developers interact with blockchain data, this project has the potential to become a foundational tool for the future of multichain development.


---

11. Current Implementation Status (Repository)
This repository currently implements a Rust-first stack:
- Core decoder: Rust (`core/chainmerge`)
- Backend API: Rust + Axum (`services/api`)
- Frontend: React + Vite (`apps/web`)

Implemented decode coverage:
- Solana: SPL Transfer + TransferChecked
- Ethereum: ERC-20 Transfer + native ETH fallback
- Cosmos: bank MsgSend
- Aptos: aptos_account::transfer payload path
- Sui: balanceChanges transfer inference
- Bitcoin: Blockstream-style tx endpoint parsing
- Polkadot: Subscan-style balances transfer parsing
- Starknet: transfer-shaped receipt event parsing

Utility endpoints:
- `GET /api/health`
- `GET /api/examples`
- `GET /api/decode`

Deployment and CI now included:
- Dockerfiles for API + web
- `docker-compose.yml`
- GitHub Actions workflow (`.github/workflows/ci.yml`)
