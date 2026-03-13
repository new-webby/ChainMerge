use serde_json::{json, Value};

use crate::chainrpc::post_json_with_failover;
use crate::errors::DecodeError;
use crate::traits::ChainDecoder;
use crate::types::{
    Action, ActionType, Chain, DecodeRequest, EventType, NormalizedEvent, NormalizedTransaction,
};

const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";

pub struct SolanaDecoder;

impl ChainDecoder for SolanaDecoder {
    fn chain_name(&self) -> &'static str {
        "solana"
    }

    fn decode(&self, request: &DecodeRequest) -> Result<NormalizedTransaction, DecodeError> {
        let tx = fetch_transaction(&request.rpc_url, &request.tx_hash)?;
        let events = extract_token_transfer_events(&tx);

        let (final_events, sender, receiver, value) = if events.is_empty() {
            let tx_sender = tx
                .pointer("/transaction/message/accountKeys/0/pubkey")
                .or_else(|| tx.pointer("/transaction/message/accountKeys/0"))
                .and_then(Value::as_str)
                .map(ToString::to_string);

            let unsupported_event = NormalizedEvent {
                event_type: EventType::Unsupported,
                token: None,
                from: tx_sender.clone(),
                to: None,
                amount: None,
                raw_program: Some("solana_generic_tx".to_string()),
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
            chain: Chain::Solana,
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
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [
            tx_hash,
            {
                "encoding": "jsonParsed",
                "maxSupportedTransactionVersion": 0,
                "commitment": "confirmed"
            }
        ]
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

fn extract_token_transfer_events(tx: &Value) -> Vec<NormalizedEvent> {
    let mut events = Vec::new();

    if let Some(instructions) = tx
        .get("transaction")
        .and_then(|t| t.get("message"))
        .and_then(|m| m.get("instructions"))
        .and_then(Value::as_array)
    {
        for instruction in instructions {
            maybe_push_transfer_event(instruction, &mut events);
        }
    }

    if let Some(inner_instruction_sets) = tx
        .get("meta")
        .and_then(|m| m.get("innerInstructions"))
        .and_then(Value::as_array)
    {
        for set in inner_instruction_sets {
            if let Some(inner_instructions) = set.get("instructions").and_then(Value::as_array) {
                for instruction in inner_instructions {
                    maybe_push_transfer_event(instruction, &mut events);
                }
            }
        }
    }

    events
}

fn maybe_push_transfer_event(instruction: &Value, events: &mut Vec<NormalizedEvent>) {
    let Some(program_id) = instruction
        .get("programId")
        .or_else(|| instruction.get("program_id"))
        .and_then(Value::as_str)
    else {
        return;
    };

    if program_id != TOKEN_PROGRAM_ID
        && program_id != TOKEN_2022_PROGRAM_ID
        && program_id != SYSTEM_PROGRAM_ID
    {
        return;
    }

    let Some(parsed) = instruction.get("parsed") else {
        return;
    };

    let Some(instruction_type) = parsed.get("type").and_then(Value::as_str) else {
        return;
    };

    if instruction_type != "transfer" && instruction_type != "transferChecked" {
        return;
    }

    let Some(info) = parsed.get("info") else {
        return;
    };

    let from = info
        .get("source")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let to = info
        .get("destination")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let amount = info
        .get("amount")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| info.get("lamports").and_then(|v| v.as_u64()).map(|v| v.to_string()))
        .or_else(|| {
            info.get("tokenAmount")
                .and_then(|t| t.get("amount"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });

    let token = if program_id == SYSTEM_PROGRAM_ID {
        Some("SOL".to_string())
    } else {
        info.get("mint")
            .and_then(Value::as_str)
            .map(ToString::to_string)
    };

    events.push(NormalizedEvent {
        event_type: EventType::TokenTransfer,
        token,
        from,
        to,
        amount,
        raw_program: Some(program_id.to_string()),
    });
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::extract_token_transfer_events;
    use crate::types::EventType;

    #[test]
    fn extracts_spl_transfer_from_outer_instruction() {
        let tx = json!({
            "transaction": {
                "message": {
                    "instructions": [
                        {
                            "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                            "parsed": {
                                "type": "transfer",
                                "info": {
                                    "source": "from_token_account",
                                    "destination": "to_token_account",
                                    "amount": "1000"
                                }
                            }
                        }
                    ]
                }
            },
            "meta": { "innerInstructions": [] }
        });

        let events = extract_token_transfer_events(&tx);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, EventType::TokenTransfer));
        assert_eq!(events[0].amount.as_deref(), Some("1000"));
        assert_eq!(events[0].from.as_deref(), Some("from_token_account"));
        assert_eq!(events[0].to.as_deref(), Some("to_token_account"));
    }

    #[test]
    fn extracts_transfer_checked_from_inner_instruction() {
        let tx = json!({
            "transaction": {
                "message": {
                    "instructions": []
                }
            },
            "meta": {
                "innerInstructions": [
                    {
                        "index": 0,
                        "instructions": [
                            {
                                "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                                "parsed": {
                                    "type": "transferChecked",
                                    "info": {
                                        "source": "from_2022",
                                        "destination": "to_2022",
                                        "mint": "mint_2022",
                                        "tokenAmount": {
                                            "amount": "420"
                                        }
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        });

        let events = extract_token_transfer_events(&tx);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].token.as_deref(), Some("mint_2022"));
        assert_eq!(events[0].amount.as_deref(), Some("420"));
    }

    #[test]
    fn extracts_native_sol_transfer() {
        let tx = json!({
            "transaction": {
                "message": {
                    "instructions": [
                        {
                            "programId": "11111111111111111111111111111111",
                            "parsed": {
                                "type": "transfer",
                                "info": {
                                    "source": "sender_address",
                                    "destination": "receiver_address",
                                    "lamports": 1000000000u64
                                }
                            }
                        }
                    ]
                }
            },
            "meta": { "innerInstructions": [] }
        });

        let events = extract_token_transfer_events(&tx);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].token.as_deref(), Some("SOL"));
        assert_eq!(events[0].amount.as_deref(), Some("1000000000"));
        assert_eq!(events[0].from.as_deref(), Some("sender_address"));
        assert_eq!(events[0].to.as_deref(), Some("receiver_address"));
    }

    #[test]
    fn ignores_non_token_program_instructions() {
        let tx = json!({
            "transaction": {
                "message": {
                    "instructions": [
                        {
                            "programId": "99999999999999999999999999999999",
                            "parsed": {
                                "type": "transfer",
                                "info": {
                                    "source": "a",
                                    "destination": "b",
                                    "amount": "1"
                                }
                            }
                        }
                    ]
                }
            },
            "meta": { "innerInstructions": [] }
        });

        let events = extract_token_transfer_events(&tx);
        assert!(events.is_empty());
    }
}
