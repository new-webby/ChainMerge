# ChainCodec RPC README

Use these RPC/base URLs with ChainCodec:

### Global Default RPC URLs
The system uses a chained failover mechanism: **Public Node 1 -> Alchemy Fallback**. If the public node fails or is rate-limited, the system automatically retries with your Alchemy endpoint.

- **ethereum**: `https://ethereum-rpc.publicnode.com` (Fallback: Alchemy)
- **solana**: `https://api.mainnet-beta.solana.com` (Fallback: Alchemy)
- **cosmos**: `https://rest.cosmos.directory/cosmoshub`
- **aptos**: `https://api.mainnet.aptoslabs.com/v1` (Fallback: Alchemy)
- **sui**: `https://fullnode.mainnet.sui.io:443` (Fallback: Alchemy)
- **bitcoin**: `https://blockstream.info/api` (Fallback: Alchemy)
- **starknet**: `https://rpc.starknet.lava.build` (Fallback: Alchemy)
- **polkadot**: `https://polkadot.api.subscan.io` (Requires Subscan API Key)

Notes:
- `polkadot` requires `POLKADOT_SUBSCAN_API_KEY`.
- `bitcoin` must be Blockstream-compatible (`/tx/{hash}` path support).

## Polkadot Subscan API Key

- `POLKADOT_SUBSCAN_API_KEY`: `a57395d3167647c7b0adc2b6f48c0fb6`
