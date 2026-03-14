use std::collections::HashMap;
use serde_json::{json, Value};
use crate::chainrpc::post_json_with_failover;
use crate::errors::DecodeError;
use crate::traits::ChainDecoder;
use crate::types::{
    Action, ActionType, Chain, DecodeRequest, EventType, NormalizedEvent, NormalizedTransaction,
};

// Demo-only fallback key so the decoder works out of the box.
// In production, override this via the POLKADOT_SUBSCAN_API_KEY env var.
const DEMO_SUBSCAN_API_KEY: &str = "a57395d3167647c7b0adc2b6f48c0fb6";

pub struct PolkadotDecoder;

impl ChainDecoder for PolkadotDecoder {
    fn chain_name(&self) -> &'static str {
        "polkadot"
    }

    fn decode(&self, request: &DecodeRequest) -> Result<NormalizedTransaction, DecodeError> {
        let payload = fetch_extrinsic_details(&request.rpc_url, &request.tx_hash)?;
        let events = extract_events(&payload);

        let sender = events.first().and_then(|e| e.from.clone());
        let receiver = events.first().and_then(|e| e.to.clone());
        let value = events.first().and_then(|e| e.amount.clone());

        let actions = events
            .iter()
            .map(|e| {
                let action_type = match e.event_type {
                    EventType::TokenTransfer => ActionType::Transfer,
                    EventType::Unsupported => ActionType::Unknown,
                };
                Action {
                    action_type,
                    from: e.from.clone(),
                    to: e.to.clone(),
                    amount: e.amount.clone(),
                    token: e.token.clone(),
                    metadata: None,
                }
            })
            .collect();

        Ok(NormalizedTransaction {
            chain: Chain::Polkadot,
            tx_hash: request.tx_hash.clone(),
            sender,
            receiver,
            value,
            events,
            actions,
        })
    }
}

fn fetch_extrinsic_details(rpc_url: &str, tx_hash: &str) -> Result<Value, DecodeError> {
    // Supports Subscan-style endpoint base, e.g. https://polkadot.api.subscan.io
    let payload = json!({ "hash": tx_hash });
    let rpc_url = format!("{}/api/scan/extrinsic", rpc_url.trim_end_matches('/'));
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    // Prefer env var, otherwise fall back to demo key.
    let api_key = std::env::var("POLKADOT_SUBSCAN_API_KEY")
        .unwrap_or_else(|_| DEMO_SUBSCAN_API_KEY.to_string());
    let trimmed = api_key.trim();
    if !trimmed.is_empty() {
        headers.insert("X-API-Key".to_string(), trimmed.to_string());
    }

    let body = post_json_with_failover(&rpc_url, &payload, Some(&headers))?;

    validate_subscan_payload(body)
}

fn validate_subscan_payload(body: Value) -> Result<Value, DecodeError> {
    if body.get("jsonrpc").is_some() {
        return Err(DecodeError::InvalidRequest(
            "polkadot decoder expects a Subscan API base URL (not a JSON-RPC endpoint)"
                .to_string(),
        ));
    }

    if body
        .get("message")
        .and_then(Value::as_str)
        .map(|m| m.to_ascii_lowercase().contains("api key"))
        .unwrap_or(false)
    {
        return Err(DecodeError::InvalidRequest(
            "Subscan API key required; set POLKADOT_SUBSCAN_API_KEY".to_string(),
        ));
    }

    if body.get("code").is_none() || body.get("data").is_none() {
        return Err(DecodeError::Rpc(
            "unexpected polkadot response format; expected Subscan payload".to_string(),
        ));
    }

    if body.get("data").is_some_and(Value::is_null) {
        return Err(DecodeError::InvalidTransactionHash);
    }

    if let Some(code) = body.get("code").and_then(Value::as_i64) {
        if code != 0 {
            if code == 403 {
                return Err(DecodeError::InvalidRequest(
                    "Subscan API key required; set POLKADOT_SUBSCAN_API_KEY".to_string(),
                ));
            }
            return Err(DecodeError::Rpc(format!("polkadot api returned error: {body}")));
        }
    }

    Ok(body)
}

fn extract_events(payload: &Value) -> Vec<NormalizedEvent> {
    let data = payload.get("data").unwrap_or(payload);
    let transfer = data.get("transfer").unwrap_or(&Value::Null);

    let module = data
        .get("call_module")
        .or_else(|| data.get("module"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    let function = data
        .get("call_module_function")
        .or_else(|| data.get("call_name"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    let raw_program = if module.is_empty() && function.is_empty() {
        None
    } else {
        Some(format!("{}_{}", module, function))
    };

    if module != "balances" || !function.contains("transfer") {
        return vec![NormalizedEvent {
            event_type: EventType::Unsupported,
            token: None,
            from: None,
            to: None,
            amount: None,
            raw_program,
        }];
    }

    let from = data
        .get("from")
        .or_else(|| transfer.get("from"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            data.get("account_display")
                .and_then(|a| a.get("address"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });
    let to = data
        .get("to")
        .or_else(|| transfer.get("to"))
        .or_else(|| data.get("dest"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let amount = find_param_value(data, "value")
        .or_else(|| find_param_value(data, "amount"))
        .or_else(|| {
            data.get("amount")
                .or_else(|| data.get("amount_raw"))
                .or_else(|| transfer.get("amount_raw"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .or_else(|| {
            transfer
                .get("amount")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .or_else(|| data.get("amount").and_then(Value::as_u64).map(|v| v.to_string()));
    let token = transfer
        .get("asset_symbol")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| Some("DOT".to_string()));

    if to.is_none() || amount.is_none() {
        return vec![NormalizedEvent {
            event_type: EventType::Unsupported,
            token: None,
            from: None,
            to: None,
            amount: None,
            raw_program,
        }];
    }

    vec![NormalizedEvent {
        event_type: EventType::TokenTransfer,
        token,
        from,
        to,
        amount,
        raw_program,
    }]
}

fn find_param_value(data: &Value, name: &str) -> Option<String> {
    let params = data.get("params")?.as_array()?;
    for param in params {
        let param_name = param.get("name").and_then(Value::as_str)?;
        if param_name != name {
            continue;
        }

        if let Some(value) = param.get("value").and_then(Value::as_str) {
            return Some(value.to_string());
        }

        if let Some(value) = param.get("value").and_then(Value::as_u64) {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{extract_events, find_param_value, validate_subscan_payload};
    use crate::errors::DecodeError;
    use crate::types::EventType;

    #[test]
    fn parses_subscan_transfer_payload() {
        let payload = json!({
            "code": 0,
            "data": {
                "call_module": "Balances",
                "call_module_function": "transfer_keep_alive",
                "from": "1from",
                "to": "1to",
                "amount": "123000000000"
            }
        });

        let events = extract_events(&payload);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].token.as_deref(), Some("DOT"));
        assert_eq!(events[0].to.as_deref(), Some("1to"));
    }

    #[test]
    fn rejects_json_rpc_endpoint_payload() {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {}
        });

        assert!(matches!(
            validate_subscan_payload(payload),
            Err(DecodeError::InvalidRequest(_))
        ));
    }

    #[test]
    fn surfaces_subscan_api_key_requirement() {
        let payload = json!({
            "code": 403,
            "message": "api key required",
            "data": {}
        });

        assert!(matches!(
            validate_subscan_payload(payload),
            Err(DecodeError::InvalidRequest(_))
        ));
    }

    #[test]
    fn surfaces_subscan_api_key_requirement_without_code_field() {
        let payload = json!({
            "message": "Subscan API strictly requires an API key"
        });

        assert!(matches!(
            validate_subscan_payload(payload),
            Err(DecodeError::InvalidRequest(_))
        ));
    }

    #[test]
    fn maps_null_data_to_invalid_transaction_hash() {
        let payload = json!({
            "code": 0,
            "message": "Success",
            "data": null
        });

        assert!(matches!(
            validate_subscan_payload(payload),
            Err(DecodeError::InvalidTransactionHash)
        ));
    }

    #[test]
    fn parses_subscan_nested_transfer_payload() {
        let payload = json!({
            "code": 0,
            "data": {
                "call_module": "balances",
                "call_module_function": "transfer_keep_alive",
                "account_display": { "address": "1from" },
                "params": [
                    { "name": "dest", "value": { "Id": "0xabc" } },
                    { "name": "value", "value": "66000000000" }
                ],
                "transfer": {
                    "from": "1from",
                    "to": "1to",
                    "asset_symbol": "DOT"
                }
            }
        });

        let events = extract_events(&payload);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].from.as_deref(), Some("1from"));
        assert_eq!(events[0].to.as_deref(), Some("1to"));
        assert_eq!(events[0].amount.as_deref(), Some("66000000000"));
        assert_eq!(events[0].token.as_deref(), Some("DOT"));
    }

    #[test]
    fn finds_param_value_from_string() {
        let payload = json!({
            "params": [
                { "name": "value", "value": "123" }
            ]
        });
        assert_eq!(find_param_value(&payload, "value").as_deref(), Some("123"));
    }

    #[test]
    fn parses_unsupported_event() {
        let payload = json!({
            "code": 0,
            "data": {
                "call_module": "timestamp",
                "call_module_function": "set",
                "params": [
                    { "name": "now", "value": 1690000000000u64 }
                ]
            }
        });

        let events = extract_events(&payload);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, EventType::Unsupported));
        assert_eq!(events[0].raw_program.as_deref(), Some("timestamp_set"));
        assert_eq!(events[0].amount, None);
        assert_eq!(events[0].token, None);
    }
}
