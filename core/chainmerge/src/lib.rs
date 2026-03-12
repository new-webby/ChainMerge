pub mod chains;
pub mod chainrpc;
pub mod errors;
pub mod normalizer;
pub mod traits;
pub mod types;

use chains::{
    aptos::AptosDecoder,
    bitcoin::BitcoinDecoder,
    cosmos::CosmosDecoder,
    ethereum::EthereumDecoder,
    polkadot::PolkadotDecoder,
    solana::SolanaDecoder,
    starknet::StarknetDecoder,
    sui::SuiDecoder,
};
use errors::DecodeError;
use traits::ChainDecoder;
use types::{Chain, DecodeRequest, NormalizedTransaction};

pub fn decode_transaction(request: &DecodeRequest) -> Result<NormalizedTransaction, DecodeError> {
    request.validate()?;

    match request.chain {
        Chain::Solana => SolanaDecoder.decode(request),
        Chain::Ethereum => EthereumDecoder.decode(request),
        Chain::Cosmos => CosmosDecoder.decode(request),
        Chain::Aptos => AptosDecoder.decode(request),
        Chain::Sui => SuiDecoder.decode(request),
        Chain::Polkadot => PolkadotDecoder.decode(request),
        Chain::Bitcoin => BitcoinDecoder.decode(request),
        Chain::Starknet => StarknetDecoder.decode(request),
    }
}
