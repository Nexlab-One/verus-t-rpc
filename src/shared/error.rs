//! Error handling module
//! 
//! This module provides centralized error handling for the application.

use thiserror::Error;
use serde_json::Value;

/// Application error types
#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("JSON serialization error: {0}")]
    Json(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Method not allowed: {method}")]
    MethodNotAllowed { method: String },

    #[error("Invalid parameters for method {method}: {reason}")]
    InvalidParameters { method: String, reason: String },

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Request too large: {size} bytes exceeds limit of {limit} bytes")]
    RequestTooLarge { size: usize, limit: usize },
}

impl AppError {
    /// Convert to JSON-RPC error response
    pub fn to_jsonrpc_error(&self) -> Value {
        let (code, message) = match self {
            AppError::MethodNotAllowed { method } => (-32601, format!("Method not found: {}", method)),
            AppError::InvalidParameters { method, reason } => (-32602, format!("Invalid parameters for {}: {}", method, reason)),
            AppError::Json(_) => (-32700, "Parse error".to_string()),
            AppError::Rpc(msg) => {
                // Try to parse as JSON-RPC error
                if let Ok(rpc_error) = serde_json::from_str::<serde_json::Value>(msg) {
                    if let (Some(code), Some(message)) = (
                        rpc_error.get("code").and_then(|c| c.as_i64()),
                        rpc_error.get("message").and_then(|m| m.as_str())
                    ) {
                        (code, message.to_string())
                    } else {
                        (-32603, msg.clone())
                    }
                } else {
                    (-32603, msg.clone())
                }
            },
            AppError::RateLimit => (-429, "Rate limit exceeded".to_string()),
            AppError::RequestTooLarge { size, limit } => (-413, format!("Request too large: {} bytes exceeds limit of {} bytes", size, limit)),
            AppError::Authentication(_) => (-401, "Authentication failed".to_string()),
            _ => (-32603, "Internal error".to_string()),
        };

        serde_json::json!({
            "error": {
                "code": code,
                "message": message
            }
        })
    }

    /// Get HTTP status code for this error
    pub fn http_status_code(&self) -> warp::http::StatusCode {
        match self {
            AppError::MethodNotAllowed { .. } => warp::http::StatusCode::METHOD_NOT_ALLOWED,
            AppError::InvalidParameters { .. } => warp::http::StatusCode::BAD_REQUEST,
            AppError::Json(_) => warp::http::StatusCode::BAD_REQUEST,
            AppError::RateLimit => warp::http::StatusCode::TOO_MANY_REQUESTS,
            AppError::RequestTooLarge { .. } => warp::http::StatusCode::PAYLOAD_TOO_LARGE,
            AppError::Authentication(_) => warp::http::StatusCode::UNAUTHORIZED,
            AppError::Rpc(_) => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            _ => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Application result type
pub type AppResult<T> = Result<T, AppError>;

// Implement warp::reject::Reject for AppError
impl warp::reject::Reject for AppError {}

impl From<config::ConfigError> for AppError {
    fn from(err: config::ConfigError) -> Self {
        AppError::Config(err.to_string())
    }
}

impl From<jsonrpc::Error> for AppError {
    fn from(err: jsonrpc::Error) -> Self {
        AppError::Rpc(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Json(err.to_string())
    }
}

impl From<jsonrpc::simple_http::Error> for AppError {
    fn from(err: jsonrpc::simple_http::Error) -> Self {
        AppError::Config(err.to_string())
    }
} 