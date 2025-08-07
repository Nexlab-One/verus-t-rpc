//! HTTP models - Infrastructure concerns
//! 
//! This module contains models that are specific to infrastructure concerns
//! like HTTP requests/responses, serialization, and external interfaces.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;
use std::time::{SystemTime, UNIX_EPOCH};

/// HTTP JSON-RPC request structure (infrastructure concern)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct JsonRpcRequest {
    /// JSON-RPC version
    #[serde(default = "default_jsonrpc_version")]
    pub jsonrpc: String,
    
    /// Method name
    #[validate(length(min = 1, max = 100))]
    pub method: String,
    
    /// Parameters (optional)
    #[serde(default)]
    pub params: Option<Value>,
    
    /// Request ID
    #[serde(default)]
    pub id: Option<Value>,
}

/// HTTP JSON-RPC response structure (infrastructure concern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version
    #[serde(default = "default_jsonrpc_version")]
    pub jsonrpc: String,
    
    /// Result (for successful responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    
    /// Error (for error responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    
    /// Request ID
    pub id: Option<Value>,
}

/// HTTP JSON-RPC error structure (infrastructure concern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i64,
    
    /// Error message
    pub message: String,
    
    /// Additional error data (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// HTTP request context for tracking and logging (infrastructure concern)
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique request ID
    pub request_id: String,
    
    /// Client IP address
    pub client_ip: String,
    
    /// User agent
    pub user_agent: Option<String>,
    
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Request method
    pub method: String,
    
    /// Request parameters (for logging)
    pub params: Option<Value>,

    /// Authorization bearer token if provided
    pub auth_token: Option<String>,
}

/// HTTP rate limit information (infrastructure concern)
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Current request count
    pub current: u32,
    
    /// Maximum allowed requests
    pub limit: u32,
    
    /// Reset time
    pub reset_time: chrono::DateTime<chrono::Utc>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(method: String, params: Option<Value>, id: Option<Value>) -> Self {
        Self {
            jsonrpc: default_jsonrpc_version(),
            method,
            params,
            id,
        }
    }
    
    /// Validate the request
    pub fn validate_request(&self) -> crate::Result<()> {
        self.validate()
            .map_err(|e| crate::shared::error::AppError::Validation(format!("Request validation failed: {}", e)))?;
        
        Ok(())
    }
    
    /// Get parameters as array
    pub fn params_as_array(&self) -> Option<Vec<Value>> {
        self.params.as_ref().and_then(|p| p.as_array()).cloned()
    }
    
    /// Get parameters as object
    pub fn params_as_object(&self) -> Option<serde_json::Map<String, Value>> {
        self.params.as_ref().and_then(|p| p.as_object()).cloned()
    }
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success(result: Value, id: Option<Value>) -> Self {
        Self {
            jsonrpc: default_jsonrpc_version(),
            result: Some(result),
            error: None,
            id,
        }
    }
    
    /// Create an error response
    pub fn error(error: JsonRpcError, id: Option<Value>) -> Self {
        Self {
            jsonrpc: default_jsonrpc_version(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

impl JsonRpcError {
    /// Create a new JSON-RPC error
    pub fn new(code: i64, message: String, data: Option<Value>) -> Self {
        Self {
            code,
            message,
            data,
        }
    }
    
    /// Create a parse error
    pub fn parse_error() -> Self {
        Self::new(-32700, "Parse error".to_string(), None)
    }
    
    /// Create an invalid request error
    pub fn invalid_request() -> Self {
        Self::new(-32600, "Invalid Request".to_string(), None)
    }
    
    /// Create a method not found error
    pub fn method_not_found(method: &str) -> Self {
        Self::new(-32601, format!("Method not found: {}", method), None)
    }
    
    /// Create an invalid params error
    pub fn invalid_params(method: &str, reason: &str) -> Self {
        Self::new(-32602, format!("Invalid parameters for {}: {}", method, reason), None)
    }
    
    /// Create an internal error
    pub fn internal_error(message: &str) -> Self {
        Self::new(-32603, format!("Internal error: {}", message), None)
    }
    
    /// Create a rate limit error
    pub fn rate_limit_error() -> Self {
        Self::new(-429, "Rate limit exceeded".to_string(), None)
    }
}

impl RequestContext {
    /// Create a new request context
    pub fn new(client_ip: String, method: String, params: Option<Value>) -> Self {
        Self {
            request_id: generate_request_id(),
            client_ip,
            user_agent: None,
            timestamp: chrono::Utc::now(),
            method,
            params,
            auth_token: None,
        }
    }
    
    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Set authorization token
    pub fn with_auth_token(mut self, auth_token: String) -> Self {
        self.auth_token = Some(auth_token);
        self
    }
}

fn default_jsonrpc_version() -> String {
    "2.0".to_string()
}

fn generate_request_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    
    format!("req_{:x}", now)
} 