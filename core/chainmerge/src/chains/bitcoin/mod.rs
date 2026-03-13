use serde_json::Value;

use crate::chainrpc::get_json_with_failover;
use crate::errors::DecodeError;
use crate::traits::ChainDecoder;
use crate::types::{
    Action, ActionType, Chain, DecodeRequest, EventType, NormalizedEvent, NormalizedTransaction,
};

pub struct BitcoinDecoder;

impl ChainDecoder for BitcoinDecoder {
    fn chain_name(&self) -> &'static str {
        "bitcoin"
    }

    fn decode(&self, request: &DecodeRequest) -> Result<NormalizedTransaction, DecodeError> {
        let tx = fetch_transaction(&request.rpc_url, &request.tx_hash)?;
        let events = extract_transfer_events(&tx);

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
            chain: Chain::Bitcoin,
            tx_hash: request.tx_hash.clone(),
            sender,
            receiver,
            value,
            events,
            actions,
        })
    }
}

fn fetch_transaction(rpc_url: &str, tx_hash: &str) -> Result<Value, DecodeError> {
    // Expects a Blockstream-compatible endpoint base (e.g. https://blockstream.info/api).
    let body = get_json_with_failover(rpc_url, &format!("/tx/{tx_hash}"), None)?;

    if body.get("txid").is_none() {
        return Err(DecodeError::Rpc("bitcoin endpoint returned unexpected payload".to_string()));
    }

    Ok(body)
}

fn extract_transfer_events(tx: &Value) -> Vec<NormalizedEvent> {
    let from = tx
        .get("vin")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|vin| vin.get("prevout"))
        .and_then(|prevout| prevout.get("scriptpubkey_address"))
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let first_output = tx
        .get("vout")
        .and_then(Value::as_array)
        .and_then(|arr| arr.iter().find(|vout| vout.get("scriptpubkey_address").is_some()));

    let to = first_output
        .and_then(|vout| vout.get("scriptpubkey_address"))
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let amount = first_output
        .and_then(|vout| vout.get("value"))
        .and_then(Value::as_u64)
        .map(|v| v.to_string());

    if to.is_none() || amount.is_none() {
        return Vec::new();
    }

    vec![NormalizedEvent {
        event_type: EventType::TokenTransfer,
        token: Some("BTC".to_string()),
        from,
        to,
        amount,
        raw_program: Some("bitcoin_utxo".to_string()),
    }]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::extract_transfer_events;

    #[test]
    fn parses_bitcoin_transfer() {
        let tx = json!({
            "txid": "abc",
            "vin": [
                {
                    "prevout": {
                        "scriptpubkey_address": "bc1from"
                    }
                }
            ],
            "vout": [
                {
                    "scriptpubkey_address": "bc1to",
                    "value": 12500
                }
            ]
        });

        let events = extract_transfer_events(&tx);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].token.as_deref(), Some("BTC"));
        assert_eq!(events[0].from.as_deref(), Some("bc1from"));
        assert_eq!(events[0].to.as_deref(), Some("bc1to"));
        assert_eq!(events[0].amount.as_deref(), Some("12500"));
    }
}
