//! HTTP responses module
//! 
//! This module contains HTTP response formatting and utilities.

use crate::{
    shared::error::AppError,
    infrastructure::http::models::{JsonRpcResponse, JsonRpcError},
};
use serde_json::Value;
use warp::http::StatusCode;
use warp::reply::{Json, WithStatus};

/// Response formatter for HTTP responses
pub struct ResponseFormatter;

impl ResponseFormatter {
    /// Format a successful JSON-RPC response
    pub fn success(result: Value, id: Option<Value>) -> Json {
        let response = JsonRpcResponse::success(result, id);
        warp::reply::json(&response)
    }

    /// Format an error JSON-RPC response
    pub fn error(error: JsonRpcError, id: Option<Value>) -> Json {
        let response = JsonRpcResponse::error(error, id);
        warp::reply::json(&response)
    }

    /// Format an error response with status code
    pub fn error_with_status(error: JsonRpcError, id: Option<Value>, status: StatusCode) -> WithStatus<Json> {
        let response = JsonRpcResponse::error(error, id);
        warp::reply::with_status(warp::reply::json(&response), status)
    }

    /// Format an application error as JSON-RPC response
    pub fn from_app_error(error: &AppError, id: Option<Value>) -> WithStatus<Json> {
        let (rpc_error, status) = match error {
            AppError::MethodNotAllowed { method } => (
                JsonRpcError::method_not_found(method),
                StatusCode::METHOD_NOT_ALLOWED
            ),
            AppError::InvalidParameters { method, reason } => (
                JsonRpcError::invalid_params(method, reason),
                StatusCode::BAD_REQUEST
            ),
            AppError::Json(_) => (
                JsonRpcError::parse_error(),
                StatusCode::BAD_REQUEST
            ),
            AppError::RateLimit => (
                JsonRpcError::rate_limit_error(),
                StatusCode::TOO_MANY_REQUESTS
            ),
            AppError::RequestTooLarge { size, limit } => (
                JsonRpcError::new(-413, format!("Request too large: {} bytes exceeds limit of {} bytes", size, limit), None),
                StatusCode::PAYLOAD_TOO_LARGE
            ),
            AppError::Authentication(_) => (
                JsonRpcError::new(-401, "Authentication failed".to_string(), None),
                StatusCode::UNAUTHORIZED
            ),
            AppError::Rpc(msg) => {
                // Try to parse as JSON-RPC error
                if let Ok(rpc_error) = serde_json::from_str::<serde_json::Value>(msg) {
                    if let (Some(code), Some(message)) = (
                        rpc_error.get("code").and_then(|c| c.as_i64()),
                        rpc_error.get("message").and_then(|m| m.as_str())
                    ) {
                        (JsonRpcError::new(code, message.to_string(), None), StatusCode::INTERNAL_SERVER_ERROR)
                    } else {
                        (JsonRpcError::internal_error(msg), StatusCode::INTERNAL_SERVER_ERROR)
                    }
                } else {
                    (JsonRpcError::internal_error(msg), StatusCode::INTERNAL_SERVER_ERROR)
                }
            },
            _ => (
                JsonRpcError::internal_error(&error.to_string()),
                StatusCode::INTERNAL_SERVER_ERROR
            ),
        };

        Self::error_with_status(rpc_error, id, status)
    }

    /// Format a health check response
    pub fn health(status: &str, version: &str) -> Json {
        let health_data = serde_json::json!({
            "status": status,
            "version": version,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        warp::reply::json(&health_data)
    }

    /// Format a metrics response
    pub fn metrics(metrics: &crate::shared::metrics::Metrics) -> Json {
        warp::reply::json(metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::Reply;

    #[test]
    fn test_success_response_creation() {
        let result = serde_json::json!({"version": "1.0.0"});
        let id = Some(serde_json::json!(1));
        
        let reply = ResponseFormatter::success(result.clone(), id.clone());
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::OK);
    }

    #[test]
    fn test_error_response_creation() {
        let error = JsonRpcError::new(-32601, "Method not found".to_string(), None);
        let id = Some(serde_json::json!(1));
        
        let reply = ResponseFormatter::error(error.clone(), id.clone());
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::OK);
    }

    #[test]
    fn test_error_with_status_creation() {
        let error = JsonRpcError::new(-32601, "Method not found".to_string(), None);
        let id = Some(serde_json::json!(1));
        let status = warp::http::StatusCode::BAD_REQUEST;
        let reply = ResponseFormatter::error_with_status(error, id.clone(), status);
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_from_app_error_method_not_allowed() {
        let error = crate::shared::error::AppError::MethodNotAllowed { method: "bad".to_string() };
        let id = Some(serde_json::json!(1));
        let reply = ResponseFormatter::from_app_error(&error, id.clone());
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn test_from_app_error_invalid_parameters() {
        let error = crate::shared::error::AppError::InvalidParameters { method: "getinfo".to_string(), reason: "bad".to_string() };
        let id = Some(serde_json::json!(1));
        let reply = ResponseFormatter::from_app_error(&error, id.clone());
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_from_app_error_rate_limit() {
        let error = crate::shared::error::AppError::RateLimit;
        let id = Some(serde_json::json!(1));
        let reply = ResponseFormatter::from_app_error(&error, id.clone());
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_from_app_error_request_too_large() {
        let error = crate::shared::error::AppError::RequestTooLarge { size: 1024 * 1024, limit: 1024 };
        let id = Some(serde_json::json!(1));
        let reply = ResponseFormatter::from_app_error(&error, id.clone());
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[test]
    fn test_from_app_error_authentication() {
        let error = crate::shared::error::AppError::Authentication("nope".to_string());
        let id = Some(serde_json::json!(1));
        let reply = ResponseFormatter::from_app_error(&error, id.clone());
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_health_response_creation() {
        let status = "healthy";
        let version = "1.0.0";
        let reply = ResponseFormatter::health(status, version);
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::OK);
    }

    #[test]
    fn test_metrics_response_creation() {
        let metrics = crate::shared::metrics::MetricsUtils::default().get_metrics();
        
        let reply = ResponseFormatter::metrics(&metrics);
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::OK);
    }
} 