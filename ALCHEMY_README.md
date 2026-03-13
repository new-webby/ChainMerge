# Alchemy RPC Configuration for ChainMerge

If the default public RPC endpoints provided in [rpcURLS.MD](rpcURLS.MD) are slow, rate-limited, or unavailable, it is highly recommended to use **Alchemy** as your RPC provider. 

These endpoints can be used in the "Custom API URL" or "RPC URL" fields within the ChainMerge Extension or Web App.

## Recommended Alchemy Endpoints

Replace `<YOUR_ALCHEMY_API_KEY>` with your actual Alchemy API Key.

### Configured Chained Endpoints
The system is now configured to use public nodes as the primary RPC source, with Alchemy as an automatic fallback if the public nodes are down or rate-limited.

| Chain | Primary (Public) | Fallback (Alchemy) |
| :--- | :--- | :--- |
| **Ethereum** | `https://ethereum-rpc.publicnode.com` | `https://eth-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq` |
| **Solana** | `https://api.mainnet-beta.solana.com` | `https://solana-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq` |
| **Aptos** | `https://api.mainnet.aptoslabs.com/v1` | `https://aptos-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq` |
| **Sui** | `https://fullnode.mainnet.sui.io:443` | `https://sui-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq` |
| **Bitcoin** | `https://blockstream.info/api` | `https://bitcoin-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq` |
| **Starknet** | `https://rpc.starknet.lava.build` | `https://starknet-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq` |

> [!NOTE]
> Alchemy provides optimized JSON-RPC for most of our supported chains. For Bitcoin, ensure your subscription includes Bitcoin API access. For Cosmos and Polkadot, please refer to [rpcURLS.MD](rpcURLS.MD) for the recommended specialized endpoints.

## How to use as Fallback

If you have not configured a custom URL in the settings, the application defaults to public nodes. To ensure higher reliability:

1.  **Get an API Key**: Sign up at [alchemy.com](https://www.alchemy.com/) and create a new project for each chain you use.
2.  **Extension**: Open the ChainMerge extension popup, go to **Settings**, and paste the relevant Alchemy URL into the **ChainMerge API URL** (or use it as the RPC URL parameter in manual decodes).
3.  **API**: When calling the API directly, pass the Alchemy URL as the `rpc_url` parameter:
    ```bash
    curl "http://localhost:8080/api/decode?chain=ethereum&hash=<TX_HASH>&rpc_url=https://eth-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq"
    ```

## Why use Alchemy?
- **Higher Rate Limits**: Public nodes often rate-limit requests during high traffic.
- **Reliability**: Alchemy provides 99.9% uptime.
- **Speed**: Optimized routing ensures faster transaction fetching for the decoder.
