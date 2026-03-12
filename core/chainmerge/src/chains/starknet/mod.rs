use num_bigint::BigUint;
use serde_json::{json, Value};

use crate::chainrpc::post_json_with_failover;
use crate::errors::DecodeError;
use crate::traits::ChainDecoder;
use crate::types::{
    Action, ActionType, Chain, DecodeRequest, EventType, NormalizedEvent, NormalizedTransaction,
};

pub struct StarknetDecoder;

impl ChainDecoder for StarknetDecoder {
    fn chain_name(&self) -> &'static str {
        "starknet"
    }

    fn decode(&self, request: &DecodeRequest) -> Result<NormalizedTransaction, DecodeError> {
        let receipt = fetch_receipt(&request.rpc_url, &request.tx_hash)?;
        let events = extract_transfer_events(&receipt);

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
            chain: Chain::Starknet,
            tx_hash: request.tx_hash.clone(),
            sender,
            receiver,
            value,
            events,
            actions,
        })
    }
}

fn fetch_receipt(rpc_url: &str, tx_hash: &str) -> Result<Value, DecodeError> {
    let rpc_tx_hash = normalize_tx_hash_for_rpc(tx_hash);

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "starknet_getTransactionReceipt",
        "params": [rpc_tx_hash]
    });

    let body = post_json_with_failover(rpc_url, &payload)?;

    if let Some(err) = body.get("error") {
        return Err(DecodeError::Rpc(format!("starknet rpc returned error: {err}")));
    }

    body.get("result")
        .cloned()
        .ok_or_else(|| DecodeError::Rpc("missing result in Starknet RPC response".to_string()))
}

fn normalize_tx_hash_for_rpc(tx_hash: &str) -> String {
    let Some(hex) = tx_hash.strip_prefix("0x") else {
        return tx_hash.to_string();
    };

    if hex.len() % 2 == 0 {
        tx_hash.to_string()
    } else {
        format!("0x0{hex}")
    }
}

fn extract_transfer_events(receipt: &Value) -> Vec<NormalizedEvent> {
    let mut out = Vec::new();

    let Some(events) = receipt.get("events").and_then(Value::as_array) else {
        return out;
    };

    for event in events {
        let Some(data) = event.get("data").and_then(Value::as_array) else {
            continue;
        };

        if data.len() < 3 {
            continue;
        }

        let from = data[0].as_str().map(ToString::to_string);
        let to = data[1].as_str().map(ToString::to_string);
        let amount = data[2].as_str().and_then(felt_hex_to_decimal);
        let token = event
            .get("from_address")
            .and_then(Value::as_str)
            .map(ToString::to_string);

        if to.is_none() || amount.is_none() {
            continue;
        }

        out.push(NormalizedEvent {
            event_type: EventType::TokenTransfer,
            token,
            from,
            to,
            amount,
            raw_program: Some("starknet_event".to_string()),
        });
        break;
    }

    out
}

fn felt_hex_to_decimal(value: &str) -> Option<String> {
    let hex = value.strip_prefix("0x")?;
    let n = BigUint::parse_bytes(hex.as_bytes(), 16)?;
    Some(n.to_str_radix(10))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{extract_transfer_events, normalize_tx_hash_for_rpc};

    #[test]
    fn parses_starknet_event_transfer_shape() {
        let receipt = json!({
            "events": [
                {
                    "from_address": "0xtoken",
                    "data": [
                        "0x1",
                        "0x2",
                        "0x64"
                    ]
                }
            ]
        });

        let events = extract_transfer_events(&receipt);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].token.as_deref(), Some("0xtoken"));
        assert_eq!(events[0].amount.as_deref(), Some("100"));
    }

    #[test]
    fn normalizes_odd_length_hash_for_rpc() {
        let input = "0x2b115e75d8961caed22948082998710ac653b088448deb421f3c2a0decd1325";
        let output = normalize_tx_hash_for_rpc(input);
        assert_eq!(
            output,
            "0x02b115e75d8961caed22948082998710ac653b088448deb421f3c2a0decd1325"
        );
    }
}
