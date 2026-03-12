use num_bigint::BigInt;
use serde_json::{json, Value};

use crate::chainrpc::post_json_with_failover;
use crate::errors::DecodeError;
use crate::traits::ChainDecoder;
use crate::types::{
    Action, ActionType, Chain, DecodeRequest, EventType, NormalizedEvent, NormalizedTransaction,
};

pub struct SuiDecoder;

impl ChainDecoder for SuiDecoder {
    fn chain_name(&self) -> &'static str {
        "sui"
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
            chain: Chain::Sui,
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
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sui_getTransactionBlock",
        "params": [
            tx_hash,
            {
                "showInput": true,
                "showEffects": true,
                "showBalanceChanges": true
            }
        ]
    });

    let body = post_json_with_failover(rpc_url, &payload)?;

    if let Some(err) = body.get("error") {
        return Err(DecodeError::Rpc(format!("sui rpc returned error: {err}")));
    }

    body.get("result")
        .cloned()
        .ok_or_else(|| DecodeError::Rpc("missing result in Sui RPC response".to_string()))
}

fn extract_transfer_events(tx: &Value) -> Vec<NormalizedEvent> {
    let balance_changes = tx
        .get("balanceChanges")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let sender = tx
        .pointer("/transaction/data/sender")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let mut negatives = Vec::new();
    let mut positives = Vec::new();

    for change in balance_changes {
        let amount_str = change
            .get("amount")
            .and_then(Value::as_str)
            .unwrap_or_default();

        let Some(amount) = BigInt::parse_bytes(amount_str.as_bytes(), 10) else {
            continue;
        };

        let owner = extract_owner_address(&change);
        let coin_type = change
            .get("coinType")
            .and_then(Value::as_str)
            .map(ToString::to_string);

        if amount.sign() == num_bigint::Sign::Minus {
            negatives.push((owner, coin_type, -amount));
        } else if amount.sign() != num_bigint::Sign::NoSign {
            positives.push((owner, coin_type, amount));
        }
    }

    for (from, coin, amount) in &negatives {
        if let Some((to, _, _)) = positives.iter().find(|(_, c, a)| c == coin && a == amount) {
            return vec![NormalizedEvent {
                event_type: EventType::TokenTransfer,
                token: coin.clone(),
                from: from.clone(),
                to: to.clone(),
                amount: Some(amount.to_string()),
                raw_program: Some("sui_balance_change".to_string()),
            }];
        }
    }

    // Fallback for reward/mint-style transactions that only expose credit deltas.
    if let Some((to, coin, amount)) = positives.iter().max_by(|a, b| a.2.cmp(&b.2)) {
        let from = match (&sender, to) {
            (Some(s), Some(t)) if s != t => Some(s.clone()),
            _ => None,
        };

        return vec![NormalizedEvent {
            event_type: EventType::TokenTransfer,
            token: coin.clone(),
            from,
            to: to.clone(),
            amount: Some(amount.to_string()),
            raw_program: Some("sui_balance_change".to_string()),
        }];
    }

    Vec::new()
}

fn extract_owner_address(change: &Value) -> Option<String> {
    let owner = change.get("owner")?;

    owner
        .get("AddressOwner")
        .or_else(|| owner.get("ObjectOwner"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::extract_transfer_events;

    #[test]
    fn parses_sui_balance_changes() {
        let tx = json!({
            "balanceChanges": [
                {
                    "owner": { "AddressOwner": "0xfrom" },
                    "coinType": "0x2::sui::SUI",
                    "amount": "-1000"
                },
                {
                    "owner": { "AddressOwner": "0xto" },
                    "coinType": "0x2::sui::SUI",
                    "amount": "1000"
                }
            ]
        });

        let events = extract_transfer_events(&tx);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].from.as_deref(), Some("0xfrom"));
        assert_eq!(events[0].to.as_deref(), Some("0xto"));
        assert_eq!(events[0].amount.as_deref(), Some("1000"));
    }

    #[test]
    fn falls_back_to_largest_credit_delta() {
        let tx = json!({
            "transaction": {
                "data": {
                    "sender": "0xsender"
                }
            },
            "balanceChanges": [
                {
                    "owner": { "AddressOwner": "0xsender" },
                    "coinType": "0x2::sui::SUI",
                    "amount": "-4841348"
                },
                {
                    "owner": { "AddressOwner": "0xsender" },
                    "coinType": "0xtoken::spring::SPRING",
                    "amount": "206810905"
                },
                {
                    "owner": { "AddressOwner": "0xsender" },
                    "coinType": "0xtoken::deep::DEEP",
                    "amount": "80730"
                }
            ]
        });

        let events = extract_transfer_events(&tx);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].token.as_deref(), Some("0xtoken::spring::SPRING"));
        assert_eq!(events[0].from, None);
        assert_eq!(events[0].to.as_deref(), Some("0xsender"));
        assert_eq!(events[0].amount.as_deref(), Some("206810905"));
    }

    #[test]
    fn uses_sender_as_from_for_credit_to_different_owner() {
        let tx = json!({
            "transaction": {
                "data": {
                    "sender": "0xfrom"
                }
            },
            "balanceChanges": [
                {
                    "owner": { "AddressOwner": "0xto" },
                    "coinType": "0xtoken::coin::COIN",
                    "amount": "12345"
                }
            ]
        });

        let events = extract_transfer_events(&tx);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].from.as_deref(), Some("0xfrom"));
        assert_eq!(events[0].to.as_deref(), Some("0xto"));
        assert_eq!(events[0].amount.as_deref(), Some("12345"));
    }
}
