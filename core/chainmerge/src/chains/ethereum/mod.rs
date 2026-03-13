use num_bigint::BigUint;
use serde_json::{json, Value};

use crate::chainrpc::post_json_with_failover;
use crate::errors::DecodeError;
use crate::traits::ChainDecoder;
use crate::types::{
    Action, ActionType, Chain, DecodeRequest, EventType, NormalizedEvent, NormalizedTransaction,
};

const ERC20_TRANSFER_TOPIC: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

pub struct EthereumDecoder;

impl ChainDecoder for EthereumDecoder {
    fn chain_name(&self) -> &'static str {
        "ethereum"
    }

    fn decode(&self, request: &DecodeRequest) -> Result<NormalizedTransaction, DecodeError> {
        let receipt = fetch_transaction_receipt(&request.rpc_url, &request.tx_hash)?;
        let mut events = extract_erc20_transfer_events(&receipt);

        if events.is_empty() {
            let transaction = fetch_transaction_by_hash(&request.rpc_url, &request.tx_hash)?;
            if let Some(native_event) = extract_native_eth_transfer_event(&transaction) {
                events.push(native_event);
            }
        }

        if events.is_empty() {
            return Err(DecodeError::UnsupportedEvent);
        }

        let sender = events.first().and_then(|e| e.from.clone());
        let receiver = events.first().and_then(|e| e.to.clone());
        let value = events.first().and_then(|e| e.amount.clone());

        let actions = events
            .iter()
            .map(|e| Action {
                action_type: ActionType::Transfer,
                from: e.from.clone(),
                to: e.to.clone(),
                amount: e.amount.clone(),
                token: e.token.clone(),
                metadata: None,
            })
            .collect();

        Ok(NormalizedTransaction {
            chain: Chain::Ethereum,
            tx_hash: request.tx_hash.clone(),
            sender,
            receiver,
            value,
            events,
            actions,
        })
    }
}

fn fetch_transaction_receipt(rpc_url: &str, tx_hash: &str) -> Result<Value, DecodeError> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_getTransactionReceipt",
        "params": [tx_hash]
    });

    let body = post_json_with_failover(rpc_url, &payload, None)?;

    if let Some(err) = body.get("error") {
        return Err(DecodeError::Rpc(format!("rpc returned error: {err}")));
    }

    let Some(result) = body.get("result") else {
        return Err(DecodeError::Rpc("missing result in RPC response".to_string()));
    };

    if result.is_null() {
        return Err(DecodeError::InvalidTransactionHash);
    }

    Ok(result.clone())
}

fn fetch_transaction_by_hash(rpc_url: &str, tx_hash: &str) -> Result<Value, DecodeError> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_getTransactionByHash",
        "params": [tx_hash]
    });

    let body = post_json_with_failover(rpc_url, &payload, None)?;

    if let Some(err) = body.get("error") {
        return Err(DecodeError::Rpc(format!("rpc returned error: {err}")));
    }

    let Some(result) = body.get("result") else {
        return Err(DecodeError::Rpc("missing result in RPC response".to_string()));
    };

    if result.is_null() {
        return Err(DecodeError::InvalidTransactionHash);
    }

    Ok(result.clone())
}

fn extract_erc20_transfer_events(receipt: &Value) -> Vec<NormalizedEvent> {
    let mut events = Vec::new();

    if let Some(logs) = receipt.get("logs").and_then(Value::as_array) {
        for log in logs {
            if let Some(event) = parse_transfer_log(log) {
                events.push(event);
            }
        }
    }

    events
}

fn extract_native_eth_transfer_event(transaction: &Value) -> Option<NormalizedEvent> {
    let from = transaction.get("from")?.as_str()?.to_ascii_lowercase();
    let to = transaction.get("to")?.as_str()?.to_ascii_lowercase();
    let amount = hex_quantity_to_decimal(transaction.get("value")?.as_str()?)?;

    if amount == "0" {
        return None;
    }

    Some(NormalizedEvent {
        event_type: EventType::TokenTransfer,
        token: Some("ETH".to_string()),
        from: Some(from),
        to: Some(to),
        amount: Some(amount),
        raw_program: Some("native_eth".to_string()),
    })
}

fn parse_transfer_log(log: &Value) -> Option<NormalizedEvent> {
    let topics = log.get("topics")?.as_array()?;
    if topics.len() < 3 {
        return None;
    }

    let topic0 = topics[0].as_str()?;
    if !topic0.eq_ignore_ascii_case(ERC20_TRANSFER_TOPIC) {
        return None;
    }

    let from = topic_to_address(topics[1].as_str()?);
    let to = topic_to_address(topics[2].as_str()?);

    let token = log
        .get("address")
        .and_then(Value::as_str)
        .map(|value| value.to_ascii_lowercase());

    let amount = log
        .get("data")
        .and_then(Value::as_str)
        .and_then(hex_quantity_to_decimal);

    Some(NormalizedEvent {
        event_type: EventType::TokenTransfer,
        token,
        from,
        to,
        amount,
        raw_program: Some("erc20".to_string()),
    })
}

fn topic_to_address(topic: &str) -> Option<String> {
    let hex = topic.strip_prefix("0x").or(Some(topic))?;
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    Some(format!("0x{}", &hex[24..]).to_ascii_lowercase())
}

fn hex_quantity_to_decimal(hex_value: &str) -> Option<String> {
    let hex = hex_value.strip_prefix("0x")?;
    if hex.is_empty() {
        return Some("0".to_string());
    }

    let value = BigUint::parse_bytes(hex.as_bytes(), 16)?;
    Some(value.to_str_radix(10))
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::{
        extract_erc20_transfer_events, extract_native_eth_transfer_event, hex_quantity_to_decimal,
        topic_to_address,
    };
    use crate::types::EventType;

    #[test]
    fn parses_transfer_log() {
        let receipt = json!({
            "logs": [
                {
                    "address": "0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                    "topics": [
                        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                        "0x0000000000000000000000001111111111111111111111111111111111111111",
                        "0x0000000000000000000000002222222222222222222222222222222222222222"
                    ],
                    "data": "0x00000000000000000000000000000000000000000000000000000000000003e8"
                }
            ]
        });

        let events = extract_erc20_transfer_events(&receipt);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, EventType::TokenTransfer));
        assert_eq!(events[0].token.as_deref(), Some("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"));
        assert_eq!(events[0].from.as_deref(), Some("0x1111111111111111111111111111111111111111"));
        assert_eq!(events[0].to.as_deref(), Some("0x2222222222222222222222222222222222222222"));
        assert_eq!(events[0].amount.as_deref(), Some("1000"));
    }

    #[test]
    fn ignores_non_transfer_topics() {
        let receipt = json!({
            "logs": [
                {
                    "address": "0x1234567890123456789012345678901234567890",
                    "topics": [
                        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "0x0000000000000000000000001111111111111111111111111111111111111111",
                        "0x0000000000000000000000002222222222222222222222222222222222222222"
                    ],
                    "data": "0x01"
                }
            ]
        });

        let events = extract_erc20_transfer_events(&receipt);
        assert!(events.is_empty());
    }

    #[test]
    fn converts_hex_quantity_to_decimal() {
        assert_eq!(hex_quantity_to_decimal("0x0").as_deref(), Some("0"));
        assert_eq!(hex_quantity_to_decimal("0x3e8").as_deref(), Some("1000"));
    }

    #[test]
    fn extracts_address_from_topic() {
        let topic = "0x000000000000000000000000abcdefabcdefabcdefabcdefabcdefabcdefabcd";
        assert_eq!(
            topic_to_address(topic).as_deref(),
            Some("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")
        );
    }

    #[test]
    fn parses_native_eth_transfer_event() {
        let transaction = json!({
            "from": "0x15566c4f33a9c279f9d3e1a5bb7589fc5a7158b1",
            "to": "0x4d3f02d54a869f0ffdd8386d092366a78a3f9f2b",
            "value": "0x38d7ea4c68000"
        });

        let event = extract_native_eth_transfer_event(&transaction).expect("should parse native transfer");
        assert_eq!(event.token.as_deref(), Some("ETH"));
        assert_eq!(event.amount.as_deref(), Some("1000000000000000"));
        assert_eq!(
            event.from.as_deref(),
            Some("0x15566c4f33a9c279f9d3e1a5bb7589fc5a7158b1")
        );
    }

    #[test]
    fn parses_erc20_transfer_from_fixture() {
        let fixture = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/ethereum/erc20_receipt.json"
        ));
        let receipt: Value =
            serde_json::from_str(fixture).expect("ethereum fixture json should be valid");

        let events = extract_erc20_transfer_events(&receipt);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].amount.as_deref(), Some("1000"));
    }
}
