use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use once_cell::sync::Lazy;
use serde_json::Value;

use crate::errors::DecodeError;

const REQUEST_TIMEOUT_SECS: u64 = 12;
const RETRIES_PER_ENDPOINT: usize = 2;
const CIRCUIT_BREAKER_THRESHOLD: u32 = 3;
const CIRCUIT_BREAKER_OPEN_SECS: u64 = 30;

static CIRCUIT_STATE: Lazy<Mutex<HashMap<String, CircuitInfo>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
pub struct RpcEndpoint {
    pub url: String,
    pub weight: u32,
}

#[derive(Debug, Clone)]
struct CircuitInfo {
    failures: u32,
    open_until: Option<Instant>,
}

pub fn parse_rpc_endpoints(rpc_urls: &str) -> Vec<RpcEndpoint> {
    rpc_urls
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|entry| {
            let mut parts = entry.split('|');
            let url = parts.next()?.trim().to_string();
            if url.is_empty() {
                return None;
            }
            let weight = parts
                .next()
                .and_then(|w| w.trim().parse::<u32>().ok())
                .filter(|w| *w > 0)
                .unwrap_or(1);

            Some(RpcEndpoint { url, weight })
        })
        .collect()
}

pub fn get_json_with_failover(rpc_urls: &str, path: &str) -> Result<Value, DecodeError> {
    let endpoints = parse_rpc_endpoints(rpc_urls);
    if endpoints.is_empty() {
        return Err(DecodeError::InvalidRequest(
            "rpc_url must include at least one endpoint".to_string(),
        ));
    }

    let ordered = weighted_order(endpoints, path.as_bytes());
    let mut errors = Vec::new();

    for endpoint in ordered {
        if is_circuit_open(&endpoint.url) {
            errors.push(format!("{} skipped (circuit open)", endpoint.url));
            continue;
        }

        let url = format!("{}{}", endpoint.url.trim_end_matches('/'), path);

        for _ in 0..RETRIES_PER_ENDPOINT {
            let result = ureq::get(&url)
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .call();

            match result {
                Ok(response) => {
                    mark_success(&endpoint.url);
                    let body: Value = response
                        .into_json()
                        .map_err(|err| DecodeError::Rpc(format!("invalid REST json: {err}")))?;
                    return Ok(body);
                }
                Err(err) => {
                    mark_failure(&endpoint.url);
                    errors.push(format!("{}: {}", endpoint.url, err));
                }
            }
        }
    }

    Err(DecodeError::Rpc(format!(
        "all rpc endpoints failed: {}",
        errors.join(" | ")
    )))
}

pub fn post_json_with_failover(rpc_urls: &str, payload: &Value) -> Result<Value, DecodeError> {
    let endpoints = parse_rpc_endpoints(rpc_urls);
    if endpoints.is_empty() {
        return Err(DecodeError::InvalidRequest(
            "rpc_url must include at least one endpoint".to_string(),
        ));
    }

    let hash_seed = payload.to_string();
    let ordered = weighted_order(endpoints, hash_seed.as_bytes());
    let mut errors = Vec::new();

    for endpoint in ordered {
        if is_circuit_open(&endpoint.url) {
            errors.push(format!("{} skipped (circuit open)", endpoint.url));
            continue;
        }

        for _ in 0..RETRIES_PER_ENDPOINT {
            let result = ureq::post(&endpoint.url)
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .set("Content-Type", "application/json")
                .send_json(payload.clone());

            match result {
                Ok(response) => {
                    mark_success(&endpoint.url);
                    let body: Value = response
                        .into_json()
                        .map_err(|err| DecodeError::Rpc(format!("invalid RPC json: {err}")))?;
                    return Ok(body);
                }
                Err(err) => {
                    mark_failure(&endpoint.url);
                    errors.push(format!("{}: {}", endpoint.url, err));
                }
            }
        }
    }

    Err(DecodeError::Rpc(format!(
        "all rpc endpoints failed: {}",
        errors.join(" | ")
    )))
}

fn weighted_order(endpoints: Vec<RpcEndpoint>, seed: &[u8]) -> Vec<RpcEndpoint> {
    let mut expanded = Vec::new();
    for ep in endpoints {
        for _ in 0..ep.weight {
            expanded.push(ep.clone());
        }
    }

    if expanded.is_empty() {
        return expanded;
    }

    let start = seed.iter().fold(0usize, |acc, b| acc.wrapping_add(*b as usize)) % expanded.len();
    expanded.rotate_left(start);

    // Keep only first appearance to avoid hammering same endpoint repeatedly in one pass.
    let mut seen: HashMap<String, ()> = HashMap::new();
    let mut unique = Vec::new();
    for ep in expanded {
        if seen.insert(ep.url.clone(), ()).is_none() {
            unique.push(ep);
        }
    }

    unique
}

fn is_circuit_open(url: &str) -> bool {
    let mut map = CIRCUIT_STATE.lock().expect("circuit mutex poisoned");
    let Some(info) = map.get_mut(url) else {
        return false;
    };

    if let Some(until) = info.open_until {
        if Instant::now() < until {
            return true;
        }
        info.open_until = None;
        info.failures = 0;
    }

    false
}

fn mark_success(url: &str) {
    let mut map = CIRCUIT_STATE.lock().expect("circuit mutex poisoned");
    map.insert(
        url.to_string(),
        CircuitInfo {
            failures: 0,
            open_until: None,
        },
    );
}

fn mark_failure(url: &str) {
    let mut map = CIRCUIT_STATE.lock().expect("circuit mutex poisoned");
    let entry = map.entry(url.to_string()).or_insert(CircuitInfo {
        failures: 0,
        open_until: None,
    });

    entry.failures = entry.failures.saturating_add(1);
    if entry.failures >= CIRCUIT_BREAKER_THRESHOLD {
        entry.open_until = Some(Instant::now() + Duration::from_secs(CIRCUIT_BREAKER_OPEN_SECS));
    }
}

#[cfg(test)]
mod tests {
    use super::parse_rpc_endpoints;

    #[test]
    fn parses_weighted_endpoints() {
        let endpoints = parse_rpc_endpoints("https://a.io|3,https://b.io|1,https://c.io");
        assert_eq!(endpoints.len(), 3);
        assert_eq!(endpoints[0].weight, 3);
        assert_eq!(endpoints[1].weight, 1);
        assert_eq!(endpoints[2].weight, 1);
    }
}
