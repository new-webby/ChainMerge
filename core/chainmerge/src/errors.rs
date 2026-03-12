use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("unsupported chain: {0}")]
    UnsupportedChain(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("invalid transaction hash")]
    InvalidTransactionHash,
    #[error("rpc error: {0}")]
    Rpc(String),
    #[error("unsupported event type")]
    UnsupportedEvent,
    #[error("internal decode error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    UnsupportedChain,
    InvalidRequest,
    InvalidTransactionHash,
    Rpc,
    UnsupportedEvent,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorDomain {
    Validation,
    Network,
    Unsupported,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalError {
    pub domain: ErrorDomain,
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
}

impl From<&DecodeError> for ErrorEnvelope {
    fn from(value: &DecodeError) -> Self {
        match value {
            DecodeError::UnsupportedChain(_) => Self {
                code: ErrorCode::UnsupportedChain,
                message: value.to_string(),
                retryable: false,
            },
            DecodeError::InvalidRequest(_) => Self {
                code: ErrorCode::InvalidRequest,
                message: value.to_string(),
                retryable: false,
            },
            DecodeError::InvalidTransactionHash => Self {
                code: ErrorCode::InvalidTransactionHash,
                message: value.to_string(),
                retryable: false,
            },
            DecodeError::Rpc(_) => Self {
                code: ErrorCode::Rpc,
                message: value.to_string(),
                retryable: true,
            },
            DecodeError::UnsupportedEvent => Self {
                code: ErrorCode::UnsupportedEvent,
                message: value.to_string(),
                retryable: false,
            },
            DecodeError::Internal(_) => Self {
                code: ErrorCode::Internal,
                message: value.to_string(),
                retryable: true,
            },
        }
    }
}

impl From<&DecodeError> for CanonicalError {
    fn from(value: &DecodeError) -> Self {
        let envelope: ErrorEnvelope = value.into();
        let domain = match value {
            DecodeError::UnsupportedChain(_) | DecodeError::UnsupportedEvent => {
                ErrorDomain::Unsupported
            }
            DecodeError::InvalidRequest(_) | DecodeError::InvalidTransactionHash => {
                ErrorDomain::Validation
            }
            DecodeError::Rpc(_) => ErrorDomain::Network,
            DecodeError::Internal(_) => ErrorDomain::Internal,
        };

        Self {
            domain,
            code: envelope.code,
            message: envelope.message,
            retryable: envelope.retryable,
        }
    }
}

pub fn map_chain_error(_chain: &str, raw_message: &str) -> DecodeError {
    let lower = raw_message.to_ascii_lowercase();

    if lower.contains("timeout") || lower.contains("temporarily unavailable") {
        return DecodeError::Rpc(raw_message.to_string());
    }
    if lower.contains("invalid hash") || lower.contains("not found") {
        return DecodeError::InvalidTransactionHash;
    }
    if lower.contains("unsupported") {
        return DecodeError::UnsupportedEvent;
    }

    DecodeError::Internal(raw_message.to_string())
}
