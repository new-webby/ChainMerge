use serde_json::Value;

use crate::chainrpc::get_json_with_failover;
use crate::errors::DecodeError;
use crate::traits::ChainDecoder;
use crate::types::{
    Action, ActionType, Chain, DecodeRequest, EventType, NormalizedEvent, NormalizedTransaction,
};

pub struct AptosDecoder;

impl ChainDecoder for AptosDecoder {
    fn chain_name(&self) -> &'static str {
        "aptos"
    }

    fn decode(&self, request: &DecodeRequest) -> Result<NormalizedTransaction, DecodeError> {
        let tx = fetch_transaction(&request.rpc_url, &request.tx_hash)?;
        let events = extract_transfer_events(&tx);
        let tx_sender = tx.get("sender").and_then(Value::as_str).map(ToString::to_string);

        let (final_events, sender, receiver, value) = if events.is_empty() {
            let unsupported_event = NormalizedEvent {
                event_type: EventType::Unsupported,
                token: None,
                from: tx_sender.clone(),
                to: None,
                amount: None,
                raw_program: Some("aptos_generic_tx".to_string()),
            };
            (vec![unsupported_event], tx_sender, None, None)
        } else {
            let s = events.first().and_then(|e| e.from.clone());
            let r = events.first().and_then(|e| e.to.clone());
            let v = events.first().and_then(|e| e.amount.clone());
            (events, s, r, v)
        };

        let actions = final_events
            .iter()
            .map(|e| Action {
                action_type: if matches!(e.event_type, EventType::Unsupported) {
                    ActionType::Unknown
                } else {
                    ActionType::Transfer
                },
                from: e.from.clone(),
                to: e.to.clone(),
                amount: e.amount.clone(),
                token: e.token.clone(),
                metadata: None,
            })
            .collect();

        Ok(NormalizedTransaction {
            chain: Chain::Aptos,
            tx_hash: request.tx_hash.clone(),
            sender,
            receiver,
            value,
            events: final_events,
            actions,
        })
    }
}

fn fetch_transaction(rpc_url: &str, tx_hash: &str) -> Result<Value, DecodeError> {
    let body = get_json_with_failover(rpc_url, &format!("/v1/transactions/by_hash/{tx_hash}"), None)?;

    if let Some(message) = body.get("message").and_then(Value::as_str) {
        return Err(DecodeError::Rpc(format!("aptos api returned error: {message}")));
    }

    Ok(body)
}

fn extract_transfer_events(tx: &Value) -> Vec<NormalizedEvent> {
    let sender = tx.get("sender").and_then(Value::as_str).map(ToString::to_string);

    let payload = tx.get("payload");
    let function = payload
        .and_then(|p| p.get("function"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    if !function.ends_with("::transfer") {
        return Vec::new();
    }

    let args = payload
        .and_then(|p| p.get("arguments"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if args.len() < 2 {
        return Vec::new();
    }

    let to = args.first().and_then(Value::as_str).map(ToString::to_string);
    let amount = args
        .get(1)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| args.get(1).and_then(Value::as_u64).map(|n| n.to_string()));

    vec![NormalizedEvent {
        event_type: EventType::TokenTransfer,
        token: Some("APT".to_string()),
        from: sender,
        to,
        amount,
        raw_program: Some("aptos_move".to_string()),
    }]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::extract_transfer_events;

    #[test]
    fn parses_aptos_transfer_payload() {
        let tx = json!({
            "sender": "0xabc",
            "payload": {
                "function": "0x1::aptos_account::transfer",
                "arguments": ["0xdef", "2500"]
            }
        });

        let events = extract_transfer_events(&tx);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].token.as_deref(), Some("APT"));
        assert_eq!(events[0].from.as_deref(), Some("0xabc"));
        assert_eq!(events[0].to.as_deref(), Some("0xdef"));
        assert_eq!(events[0].amount.as_deref(), Some("2500"));
    }
}
