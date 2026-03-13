use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::Path,
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{Path as AxumPath, Query, Request, State},
    http::header,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chainmerge::{
    decode_transaction,
    errors::{DecodeError, ErrorCode, ErrorEnvelope},
    types::{Chain, DecodeRequest, NormalizedTransaction},
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    metrics: Arc<Metrics>,
    rate_window: Arc<Mutex<HashMap<String, RateCounter>>>,
    api_key: Option<String>,
    rate_limit_per_min: u32,
    index_db_path: String,
}

#[derive(Debug, Clone, Copy)]
struct RateCounter {
    minute_bucket: u64,
    count: u32,
}

#[derive(Default)]
struct Metrics {
    total_requests: AtomicU64,
    decode_requests: AtomicU64,
    decode_success: AtomicU64,
    decode_errors: AtomicU64,
    indexed_transactions: AtomicU64,
}

#[derive(Debug, Deserialize)]
struct DecodeQuery {
    chain: String,
    hash: String,
    rpc_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RecentQuery {
    limit: Option<u32>,
}

#[derive(Debug, Serialize)]
struct DecodeHttpResponse {
    decoded: NormalizedTransaction,
}

#[derive(Debug, Serialize)]
struct IndexedHttpResponse {
    indexed: bool,
    decoded: NormalizedTransaction,
}

#[derive(Debug, Serialize)]
struct IndexedListHttpResponse {
    items: Vec<NormalizedTransaction>,
}

#[derive(Debug, Serialize)]
struct ErrorHttpResponse {
    error: ErrorEnvelope,
}

#[derive(Debug, Serialize)]
struct HealthHttpResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Debug, Serialize)]
struct ExampleTx {
    chain: &'static str,
    tx_hash: &'static str,
    note: &'static str,
}

#[derive(Debug, Serialize)]
struct ExamplesHttpResponse {
    examples: Vec<ExampleTx>,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    envelope: ErrorEnvelope,
}

impl ApiError {
    fn from_decode(err: DecodeError) -> Self {
        let status = match err {
            DecodeError::UnsupportedChain(_) | DecodeError::InvalidRequest(_) => {
                StatusCode::BAD_REQUEST
            }
            DecodeError::InvalidTransactionHash => StatusCode::UNPROCESSABLE_ENTITY,
            DecodeError::UnsupportedEvent => StatusCode::NOT_IMPLEMENTED,
            DecodeError::Rpc(_) => StatusCode::BAD_GATEWAY,
            DecodeError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let envelope: ErrorEnvelope = (&err).into();
        Self { status, envelope }
    }

    fn unauthorized(message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            envelope: ErrorEnvelope {
                code: ErrorCode::InvalidRequest,
                message: message.to_string(),
                retryable: false,
            },
        }
    }

    fn rate_limited(message: &str) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            envelope: ErrorEnvelope {
                code: ErrorCode::InvalidRequest,
                message: message.to_string(),
                retryable: true,
            },
        }
    }

    fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            envelope: ErrorEnvelope {
                code: ErrorCode::InvalidRequest,
                message: message.to_string(),
                retryable: false,
            },
        }
    }

    fn internal(message: &str) -> Self {
        Self::from_decode(DecodeError::Internal(message.to_string()))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorHttpResponse {
                error: self.envelope,
            }),
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() {
    // Load local .env (searches current dir and parents) so keys are available
    // even when the process is launched without shell-level exports.
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chainmerge_api=info".into()),
        )
        .init();

    if std::env::var("POLKADOT_SUBSCAN_API_KEY")
        .map(|v| v.trim().is_empty())
        .unwrap_or(true)
    {
        warn!("POLKADOT_SUBSCAN_API_KEY is not set; Polkadot decode may fail");
    }

    let app = app();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("invalid HOST/PORT combination");

    info!(%addr, "chainmerge-api listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind tcp listener");

    axum::serve(listener, app)
        .await
        .expect("server error");
}

fn app() -> Router {
    let api_key = std::env::var("API_KEY").ok().filter(|v| !v.trim().is_empty());
    let rate_limit_per_min = std::env::var("RATE_LIMIT_PER_MIN")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(120);
    let index_db_path = std::env::var("INDEX_DB_PATH").unwrap_or_else(|_| "data/chainindex.db".to_string());

    app_with_config(api_key, rate_limit_per_min, index_db_path)
}

fn app_with_config(api_key: Option<String>, rate_limit_per_min: u32, index_db_path: String) -> Router {
    init_index_db(&index_db_path).expect("failed to initialize index database");

    let state = AppState {
        metrics: Arc::new(Metrics::default()),
        rate_window: Arc::new(Mutex::new(HashMap::new())),
        api_key,
        rate_limit_per_min,
        index_db_path,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers([header::CONTENT_TYPE, header::HeaderName::from_static("x-api-key")]);

    Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/examples", get(examples_handler))
        .route("/api/metrics", get(metrics_handler))
        .route("/api/decode", get(decode_handler))
        .route("/api/index/decode", get(index_decode_handler))
        .route("/api/index/{chain}/{hash}", get(index_lookup_handler))
        .route("/api/index/recent", get(index_recent_handler))
        .layer(middleware::from_fn_with_state(state.clone(), guard_middleware))
        .layer(cors)
        .with_state(state)
}

fn init_index_db(path: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS decoded_transactions (
            chain TEXT NOT NULL,
            tx_hash TEXT NOT NULL,
            payload TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY(chain, tx_hash)
        );
        CREATE INDEX IF NOT EXISTS idx_decoded_transactions_updated_at ON decoded_transactions(updated_at DESC);
        ",
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

async fn guard_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    state.metrics.total_requests.fetch_add(1, Ordering::Relaxed);

    if let Some(required_key) = state.api_key.as_ref() {
        let provided = req
            .headers()
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();

        if provided != required_key {
            return Err(ApiError::unauthorized("missing or invalid x-api-key"));
        }
    }

    let client_key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string)
        .unwrap_or_else(|| "anonymous".to_string());

    let current_minute = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ApiError::internal("time error"))?
        .as_secs()
        / 60;

    let mut map = state.rate_window.lock().await;
    let entry = map.entry(client_key).or_insert(RateCounter {
        minute_bucket: current_minute,
        count: 0,
    });

    if entry.minute_bucket != current_minute {
        entry.minute_bucket = current_minute;
        entry.count = 0;
    }

    entry.count = entry.count.saturating_add(1);
    if entry.count > state.rate_limit_per_min {
        return Err(ApiError::rate_limited("rate limit exceeded"));
    }

    Ok(next.run(req).await)
}

async fn health_handler() -> Json<HealthHttpResponse> {
    Json(HealthHttpResponse {
        status: "ok",
        service: "chainmerge-api",
    })
}

async fn examples_handler() -> Json<ExamplesHttpResponse> {
    Json(ExamplesHttpResponse {
        examples: vec![
            ExampleTx {
                chain: "ethereum",
                tx_hash: "0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad",
                note: "ERC-20 transfer",
            },
            ExampleTx {
                chain: "ethereum",
                tx_hash: "0x5db45209923531658781b4a5ea73bde7193e7f0991595ad5af80121764afb8b4",
                note: "Native ETH transfer",
            },
            ExampleTx {
                chain: "cosmos",
                tx_hash: "6C166D13D94E626BB6477398B1D0AEB9B5C595D0B0DA8FC7AD2191BEEF024A27",
                note: "Cosmos bank MsgSend",
            },
        ],
    })
}

async fn metrics_handler(State(state): State<AppState>) -> String {
    format!(
        "chainmerge_requests_total {}\nchainmerge_decode_requests_total {}\nchainmerge_decode_success_total {}\nchainmerge_decode_errors_total {}\nchainmerge_indexed_transactions_total {}\n",
        state.metrics.total_requests.load(Ordering::Relaxed),
        state.metrics.decode_requests.load(Ordering::Relaxed),
        state.metrics.decode_success.load(Ordering::Relaxed),
        state.metrics.decode_errors.load(Ordering::Relaxed),
        state.metrics.indexed_transactions.load(Ordering::Relaxed)
    )
}

async fn decode_handler(
    State(state): State<AppState>,
    Query(query): Query<DecodeQuery>,
) -> Result<Json<DecodeHttpResponse>, ApiError> {
    state.metrics.decode_requests.fetch_add(1, Ordering::Relaxed);

    let chain = Chain::from_str(query.chain.trim()).map_err(ApiError::from_decode)?;
    let rpc_url = resolve_rpc_url(chain, query.rpc_url).map_err(ApiError::from_decode)?;

    let request = DecodeRequest {
        chain,
        tx_hash: query.hash,
        rpc_url,
    };

    match decode_transaction(&request) {
        Ok(decoded) => {
            state.metrics.decode_success.fetch_add(1, Ordering::Relaxed);
            persist_indexed_transaction(&state.index_db_path, &decoded)?;
            state.metrics.indexed_transactions.fetch_add(1, Ordering::Relaxed);
            Ok(Json(DecodeHttpResponse { decoded }))
        }
        Err(err) => {
            state.metrics.decode_errors.fetch_add(1, Ordering::Relaxed);
            Err(ApiError::from_decode(err))
        }
    }
}

fn resolve_rpc_url(chain: Chain, request_rpc_url: Option<String>) -> Result<String, DecodeError> {
    if let Some(url) = request_rpc_url {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    let chain_env_key = match chain {
        Chain::Ethereum => "CHAINMERGE_RPC_URL_ETHEREUM",
        Chain::Solana => "CHAINMERGE_RPC_URL_SOLANA",
        Chain::Cosmos => "CHAINMERGE_RPC_URL_COSMOS",
        Chain::Aptos => "CHAINMERGE_RPC_URL_APTOS",
        Chain::Sui => "CHAINMERGE_RPC_URL_SUI",
        Chain::Polkadot => "CHAINMERGE_RPC_URL_POLKADOT",
        Chain::Bitcoin => "CHAINMERGE_RPC_URL_BITCOIN",
        Chain::Starknet => "CHAINMERGE_RPC_URL_STARKNET",
    };

    if let Ok(url) = std::env::var(chain_env_key) {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    if let Ok(url) = std::env::var("CHAINMERGE_RPC_URL") {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    let default_url = match chain {
        Chain::Ethereum => "https://ethereum-rpc.publicnode.com,https://eth-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq",
        Chain::Solana => "https://api.mainnet-beta.solana.com,https://solana-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq",
        Chain::Cosmos => "https://rest.cosmos.directory/cosmoshub",
        Chain::Aptos => "https://api.mainnet.aptoslabs.com/v1,https://aptos-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq",
        Chain::Sui => "https://fullnode.mainnet.sui.io:443,https://sui-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq",
        Chain::Polkadot => "https://polkadot.api.subscan.io",
        Chain::Bitcoin => "https://blockstream.info/api,https://bitcoin-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq",
        Chain::Starknet => "https://rpc.starknet.lava.build,https://starknet-mainnet.g.alchemy.com/v2/6Wc9vExpYDvd9UD6D3Bfq",
    };

    Ok(default_url.to_string())
}

async fn index_decode_handler(
    State(state): State<AppState>,
    Query(query): Query<DecodeQuery>,
) -> Result<Json<IndexedHttpResponse>, ApiError> {
    let Json(response) = decode_handler(State(state), Query(query)).await?;
    Ok(Json(IndexedHttpResponse {
        indexed: true,
        decoded: response.decoded,
    }))
}

async fn index_lookup_handler(
    State(state): State<AppState>,
    AxumPath((chain, hash)): AxumPath<(String, String)>,
) -> Result<Json<DecodeHttpResponse>, ApiError> {
    let found = lookup_indexed_transaction(&state.index_db_path, &chain, &hash)?;

    let Some(decoded) = found else {
        return Err(ApiError::not_found("indexed transaction not found"));
    };

    Ok(Json(DecodeHttpResponse { decoded }))
}

async fn index_recent_handler(
    State(state): State<AppState>,
    Query(query): Query<RecentQuery>,
) -> Result<Json<IndexedListHttpResponse>, ApiError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 200);
    let items = recent_indexed_transactions(&state.index_db_path, limit)?;
    Ok(Json(IndexedListHttpResponse { items }))
}

fn persist_indexed_transaction(path: &str, tx: &NormalizedTransaction) -> Result<(), ApiError> {
    let payload = serde_json::to_string(tx).map_err(|e| ApiError::internal(&e.to_string()))?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ApiError::internal("time error"))?
        .as_secs() as i64;

    let conn = Connection::open(path).map_err(|e| ApiError::internal(&e.to_string()))?;
    conn.execute(
        "INSERT INTO decoded_transactions (chain, tx_hash, payload, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(chain, tx_hash) DO UPDATE SET
           payload=excluded.payload,
           updated_at=excluded.updated_at",
        params![
            tx.chain.as_str().to_ascii_lowercase(),
            tx.tx_hash.to_ascii_lowercase(),
            payload,
            now
        ],
    )
    .map_err(|e| ApiError::internal(&e.to_string()))?;

    Ok(())
}

fn lookup_indexed_transaction(
    path: &str,
    chain: &str,
    hash: &str,
) -> Result<Option<NormalizedTransaction>, ApiError> {
    let conn = Connection::open(path).map_err(|e| ApiError::internal(&e.to_string()))?;
    let mut stmt = conn
        .prepare("SELECT payload FROM decoded_transactions WHERE chain = ?1 AND tx_hash = ?2 LIMIT 1")
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    let mut rows = stmt
        .query(params![chain.to_ascii_lowercase(), hash.to_ascii_lowercase()])
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    let Some(row) = rows.next().map_err(|e| ApiError::internal(&e.to_string()))? else {
        return Ok(None);
    };

    let payload: String = row.get(0).map_err(|e| ApiError::internal(&e.to_string()))?;
    let tx: NormalizedTransaction =
        serde_json::from_str(&payload).map_err(|e| ApiError::internal(&e.to_string()))?;

    Ok(Some(tx))
}

fn recent_indexed_transactions(path: &str, limit: u32) -> Result<Vec<NormalizedTransaction>, ApiError> {
    let conn = Connection::open(path).map_err(|e| ApiError::internal(&e.to_string()))?;
    let mut stmt = conn
        .prepare(
            "SELECT payload FROM decoded_transactions
             ORDER BY updated_at DESC
             LIMIT ?1",
        )
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    let rows = stmt
        .query_map(params![limit as i64], |row| row.get::<_, String>(0))
        .map_err(|e| ApiError::internal(&e.to_string()))?;

    let mut items = Vec::new();
    for row in rows {
        let payload = row.map_err(|e| ApiError::internal(&e.to_string()))?;
        let tx: NormalizedTransaction =
            serde_json::from_str(&payload).map_err(|e| ApiError::internal(&e.to_string()))?;
        items.push(tx);
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chainmerge::types::Chain;
    use tower::util::ServiceExt;

    use super::{app_with_config, resolve_rpc_url};

    fn test_db_path() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        format!("/tmp/chainmerge-index-test-{nanos}.db")
    }

    #[tokio::test]
    async fn decode_route_rejects_unknown_chain() {
        let response = app_with_config(None, 120, test_db_path())
            .oneshot(
                Request::builder()
                    .uri("/api/decode?chain=unknown&hash=abc&rpc_url=https://rpc.example")
                    .body(Body::empty())
                    .expect("request build should succeed"),
            )
            .await
            .expect("request should return response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn decode_route_can_use_internal_rpc_defaults() {
        let response = app_with_config(None, 120, test_db_path())
            .oneshot(
                Request::builder()
                    .uri("/api/decode?chain=ethereum&hash=not-a-valid-eth-hash")
                    .body(Body::empty())
                    .expect("request build should succeed"),
            )
            .await
            .expect("request should return response");

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn decode_route_rejects_invalid_hash_format() {
        let response = app_with_config(None, 120, test_db_path())
            .oneshot(
                Request::builder()
                    .uri("/api/decode?chain=ethereum&hash=not-a-valid-eth-hash&rpc_url=https://rpc.example")
                    .body(Body::empty())
                    .expect("request build should succeed"),
            )
            .await
            .expect("request should return response");

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn health_route_returns_ok() {
        let response = app_with_config(None, 120, test_db_path())
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .expect("request build should succeed"),
            )
            .await
            .expect("request should return response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn examples_route_returns_ok() {
        let response = app_with_config(None, 120, test_db_path())
            .oneshot(
                Request::builder()
                    .uri("/api/examples")
                    .body(Body::empty())
                    .expect("request build should succeed"),
            )
            .await
            .expect("request should return response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn metrics_route_returns_ok() {
        let response = app_with_config(None, 120, test_db_path())
            .oneshot(
                Request::builder()
                    .uri("/api/metrics")
                    .body(Body::empty())
                    .expect("request build should succeed"),
            )
            .await
            .expect("request should return response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn auth_guard_rejects_missing_key() {
        let response = app_with_config(Some("secret".to_string()), 120, test_db_path())
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .expect("request build should succeed"),
            )
            .await
            .expect("request should return response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rate_limit_rejects_after_threshold() {
        let app = app_with_config(None, 1, test_db_path());

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .expect("request build should succeed"),
            )
            .await
            .expect("request should return response");
        assert_eq!(first.status(), StatusCode::OK);

        let second = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .expect("request build should succeed"),
            )
            .await
            .expect("request should return response");
        assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn resolve_rpc_url_uses_chain_default() {
        std::env::remove_var("CHAINMERGE_RPC_URL_ETHEREUM");
        std::env::remove_var("CHAINMERGE_RPC_URL");

        let resolved =
            resolve_rpc_url(Chain::Ethereum, None).expect("rpc url resolution should work");
        assert_eq!(resolved, "https://ethereum-rpc.publicnode.com");
    }
}
