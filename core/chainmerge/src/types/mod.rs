use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::errors::DecodeError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Chain {
    Solana,
    Ethereum,
    Cosmos,
    Aptos,
    Sui,
    Polkadot,
    Bitcoin,
    Starknet,
}

impl Chain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Chain::Solana => "solana",
            Chain::Ethereum => "ethereum",
            Chain::Cosmos => "cosmos",
            Chain::Aptos => "aptos",
            Chain::Sui => "sui",
            Chain::Polkadot => "polkadot",
            Chain::Bitcoin => "bitcoin",
            Chain::Starknet => "starknet",
        }
    }
}

impl FromStr for Chain {
    type Err = DecodeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "solana" => Ok(Chain::Solana),
            "ethereum" => Ok(Chain::Ethereum),
            "cosmos" => Ok(Chain::Cosmos),
            "aptos" => Ok(Chain::Aptos),
            "sui" => Ok(Chain::Sui),
            "polkadot" => Ok(Chain::Polkadot),
            "bitcoin" => Ok(Chain::Bitcoin),
            "starknet" => Ok(Chain::Starknet),
            other => Err(DecodeError::UnsupportedChain(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    TokenTransfer,
    Unsupported,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::TokenTransfer => "token_transfer",
            EventType::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedEvent {
    pub event_type: EventType,
    pub token: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub amount: Option<String>,
    pub raw_program: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Transfer,
    Swap,
    NftTransfer,
    Stake,
    Bridge,
    Unknown,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionType::Transfer => "transfer",
            ActionType::Swap => "swap",
            ActionType::NftTransfer => "nft_transfer",
            ActionType::Stake => "stake",
            ActionType::Bridge => "bridge",
            ActionType::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,

    pub from: Option<String>,
    pub to: Option<String>,

    /// Primary amount involved in the action (e.g. sent amount)
    pub amount: Option<String>,

    /// Primary token/asset identifier (e.g. ERC-20 address, mint, denom)
    pub token: Option<String>,

    /// Optional extra data for richer actions (swap routes, pool, etc.)
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedTransaction {
    pub chain: Chain,
    pub tx_hash: String,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub value: Option<String>,
    pub events: Vec<NormalizedEvent>,

    /// High-level semantics derived from events/logs.
    #[serde(default)]
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodeRequest {
    pub chain: Chain,
    pub tx_hash: String,
    pub rpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodeResponse {
    pub transaction: NormalizedTransaction,
}

impl DecodeRequest {
    pub fn validate(&self) -> Result<(), DecodeError> {
        if self.rpc_url.trim().is_empty() {
            return Err(DecodeError::InvalidRequest(
                "rpc_url cannot be empty".to_string(),
            ));
        }

        let tx_hash = self.tx_hash.trim();
        if tx_hash.is_empty() {
            return Err(DecodeError::InvalidTransactionHash);
        }

        if !is_valid_tx_hash(self.chain, tx_hash) {
            return Err(DecodeError::InvalidTransactionHash);
        }

        Ok(())
    }
}

fn is_valid_tx_hash(chain: Chain, tx_hash: &str) -> bool {
    match chain {
        Chain::Ethereum => {
            tx_hash.len() == 66
                && tx_hash.starts_with("0x")
                && tx_hash[2..].chars().all(|ch| ch.is_ascii_hexdigit())
        }
        Chain::Starknet => {
            tx_hash.starts_with("0x")
                && (1..=64).contains(&(tx_hash.len().saturating_sub(2)))
                && tx_hash[2..].chars().all(|ch| ch.is_ascii_hexdigit())
        }
        Chain::Bitcoin => tx_hash.len() == 64 && tx_hash.chars().all(|ch| ch.is_ascii_hexdigit()),
        Chain::Solana | Chain::Cosmos | Chain::Aptos | Chain::Sui | Chain::Polkadot => {
            (32..=128).contains(&tx_hash.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Chain, DecodeRequest};
    use crate::errors::DecodeError;

    #[test]
    fn parse_chain() {
        assert_eq!("solana".parse::<Chain>().ok(), Some(Chain::Solana));
        assert!("unknown".parse::<Chain>().is_err());
    }

    #[test]
    fn validate_eth_hash() {
        let request = DecodeRequest {
            chain: Chain::Ethereum,
            tx_hash: format!("0x{}", "a".repeat(64)),
            rpc_url: "https://rpc.example".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn reject_invalid_hash() {
        let request = DecodeRequest {
            chain: Chain::Bitcoin,
            tx_hash: "invalid".to_string(),
            rpc_url: "https://rpc.example".to_string(),
        };
        assert!(matches!(
            request.validate(),
            Err(DecodeError::InvalidTransactionHash)
        ));
    }

    #[test]
    fn validate_starknet_short_hex_hash() {
        let request = DecodeRequest {
            chain: Chain::Starknet,
            tx_hash: "0x2b115e75d8961caed22948082998710ac653b088448deb421f3c2a0decd1325"
                .to_string(),
            rpc_url: "https://rpc.example".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn reject_starknet_hash_too_long() {
        let request = DecodeRequest {
            chain: Chain::Starknet,
            tx_hash: format!("0x{}", "a".repeat(65)),
            rpc_url: "https://rpc.example".to_string(),
        };
        assert!(matches!(
            request.validate(),
            Err(DecodeError::InvalidTransactionHash)
        ));
    }
}
