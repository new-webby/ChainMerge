use serde_json::Value;

use crate::errors::DecodeError;
use crate::traits::ChainDecoder;
use crate::types::{
    Action, ActionType, Chain, DecodeRequest, EventType, NormalizedEvent, NormalizedTransaction,
};

pub struct CosmosDecoder;

impl ChainDecoder for CosmosDecoder {
    fn chain_name(&self) -> &'static str {
        "cosmos"
    }

    fn decode(&self, request: &DecodeRequest) -> Result<NormalizedTransaction, DecodeError> {
        let tx = fetch_transaction(&request.rpc_url, &request.tx_hash)?;
        let events = extract_msg_send_events(&tx);

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
            chain: Chain::Cosmos,
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
    let url = format!(
        "{}/cosmos/tx/v1beta1/txs/{}",
        rpc_url.trim_end_matches('/'),
        tx_hash
    );

    let response = ureq::get(&url)
        .call()
        .map_err(|err| DecodeError::Rpc(err.to_string()))?;

    let body: Value = response
        .into_json()
        .map_err(|err| DecodeError::Rpc(format!("invalid REST json: {err}")))?;

    if body.get("code").is_some() && body.get("message").is_some() {
        return Err(DecodeError::Rpc(format!("cosmos api returned error: {body}")));
    }

    Ok(body)
}

fn extract_msg_send_events(tx: &Value) -> Vec<NormalizedEvent> {
    let mut events = Vec::new();

    let Some(messages) = tx
        .get("tx")
        .and_then(|t| t.get("body"))
        .and_then(|b| b.get("messages"))
        .and_then(Value::as_array)
    else {
        return events;
    };

    for message in messages {
        let Some(msg_type) = message.get("@type").and_then(Value::as_str) else {
            continue;
        };

        if msg_type != "/cosmos.bank.v1beta1.MsgSend" {
            continue;
        }

        let from = message
            .get("from_address")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let to = message
            .get("to_address")
            .and_then(Value::as_str)
            .map(ToString::to_string);

        if let Some(amounts) = message.get("amount").and_then(Value::as_array) {
            for coin in amounts {
                let denom = coin
                    .get("denom")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let amount = coin
                    .get("amount")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);

                events.push(NormalizedEvent {
                    event_type: EventType::TokenTransfer,
                    token: denom,
                    from: from.clone(),
                    to: to.clone(),
                    amount,
                    raw_program: Some("cosmos_bank".to_string()),
                });
            }
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::extract_msg_send_events;
    use crate::types::EventType;

    #[test]
    fn parses_msg_send_event() {
        let tx = json!({
            "tx": {
                "body": {
                    "messages": [
                        {
                            "@type": "/cosmos.bank.v1beta1.MsgSend",
                            "from_address": "cosmos1fromaddressxxxxxxxxxxxxxx",
                            "to_address": "cosmos1toaddressxxxxxxxxxxxxxxxx",
                            "amount": [
                                { "denom": "uatom", "amount": "12345" }
                            ]
                        }
                    ]
                }
            }
        });

        let events = extract_msg_send_events(&tx);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, EventType::TokenTransfer));
        assert_eq!(events[0].token.as_deref(), Some("uatom"));
        assert_eq!(events[0].amount.as_deref(), Some("12345"));
        assert_eq!(
            events[0].from.as_deref(),
            Some("cosmos1fromaddressxxxxxxxxxxxxxx")
        );
    }

    #[test]
    fn parses_multi_coin_msg_send() {
        let tx = json!({
            "tx": {
                "body": {
                    "messages": [
                        {
                            "@type": "/cosmos.bank.v1beta1.MsgSend",
                            "from_address": "cosmos1from",
                            "to_address": "cosmos1to",
                            "amount": [
                                { "denom": "uatom", "amount": "10" },
                                { "denom": "uosmo", "amount": "20" }
                            ]
                        }
                    ]
                }
            }
        });

        let events = extract_msg_send_events(&tx);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].token.as_deref(), Some("uatom"));
        assert_eq!(events[1].token.as_deref(), Some("uosmo"));
    }

    #[test]
    fn ignores_non_bank_messages() {
        let tx = json!({
            "tx": {
                "body": {
                    "messages": [
                        {
                            "@type": "/cosmos.staking.v1beta1.MsgDelegate",
                            "delegator_address": "cosmos1..."
                        }
                    ]
                }
            }
        });

        let events = extract_msg_send_events(&tx);
        assert!(events.is_empty());
    }

    #[test]
    fn parses_msg_send_from_fixture() {
        let fixture = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/cosmos/msg_send_tx.json"
        ));
        let tx: Value = serde_json::from_str(fixture).expect("cosmos fixture json should be valid");

        let events = extract_msg_send_events(&tx);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].token.as_deref(), Some("uatom"));
        assert_eq!(events[1].token.as_deref(), Some("uosmo"));
    }
}
