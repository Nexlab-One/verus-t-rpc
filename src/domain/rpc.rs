//! RPC domain logic - Core RPC business rules and models

use crate::shared::error::AppResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// RPC method definition with business rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMethod {
    /// Method name
    pub name: String,
    
    /// Method description
    pub description: String,
    
    /// Whether this method is read-only
    pub read_only: bool,
    
    /// Required permissions
    pub required_permissions: Vec<String>,
    
    /// Parameter validation rules
    pub parameter_rules: Vec<ParameterRule>,
}

/// Parameter validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRule {
    /// Parameter index
    pub index: usize,
    
    /// Parameter name
    pub name: String,
    
    /// Parameter type
    pub param_type: ParameterType,
    
    /// Whether parameter is required
    pub required: bool,
    
    /// Validation constraints
    pub constraints: Vec<Constraint>,
}

/// Parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Any,
}

/// Validation constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    MinLength(usize),
    MaxLength(usize),
    MinValue(f64),
    MaxValue(f64),
    Pattern(String),
    Custom(String),
}

/// RPC request with domain validation
#[derive(Debug, Clone)]
pub struct RpcRequest {
    /// Method name
    pub method: String,
    
    /// Parameters
    pub parameters: Option<Value>,
    
    /// Request ID
    pub id: Option<Value>,
    
    /// Client information
    pub client_info: ClientInfo,
}

/// Client information for request tracking
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Client IP address
    pub ip_address: String,
    
    /// User agent
    pub user_agent: Option<String>,
    
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// RPC response with domain logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    
    /// Result (for successful responses)
    pub result: Option<Value>,
    
    /// Error (for error responses)
    pub error: Option<RpcError>,
    
    /// Request ID
    pub id: Option<Value>,
}

/// RPC error with domain context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    /// Error code
    pub code: i64,
    
    /// Error message
    pub message: String,
    
    /// Additional error data
    pub data: Option<Value>,
    
    /// Error context for debugging
    pub context: Option<ErrorContext>,
}

/// Error context for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Method that caused the error
    pub method: String,
    
    /// Parameters that caused the error
    pub parameters: Option<Value>,
    
    /// Timestamp when error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl RpcRequest {
    /// Create a new RPC request
    pub fn new(method: String, parameters: Option<Value>, id: Option<Value>, client_info: ClientInfo) -> Self {
        Self {
            method,
            parameters,
            id,
            client_info,
        }
    }
    
    /// Validate the request against business rules
    pub fn validate(&self) -> AppResult<()> {
        // Basic validation
        if self.method.is_empty() {
            return Err(crate::shared::error::AppError::Validation("Method name cannot be empty".to_string()));
        }
        
        if self.method.len() > 100 {
            return Err(crate::shared::error::AppError::Validation("Method name too long".to_string()));
        }
        
        // Validate parameters if present
        if let Some(params) = &self.parameters {
            self.validate_parameters(params)?;
        }
        
        Ok(())
    }
    
    /// Validate parameters against business rules
    fn validate_parameters(&self, params: &Value) -> AppResult<()> {
        // Ensure parameters is an array
        if !params.is_array() {
            return Err(crate::shared::error::AppError::Validation("Parameters must be an array".to_string()));
        }
        
        // Check parameter count limits
        if let Some(array) = params.as_array() {
            if array.len() > 100 {
                return Err(crate::shared::error::AppError::Validation("Too many parameters".to_string()));
            }
        }
        
        Ok(())
    }
}

impl RpcResponse {
    /// Create a successful response
    pub fn success(result: Value, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }
    
    /// Create an error response
    pub fn error(error: RpcError, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

impl RpcError {
    /// Create a new RPC error
    pub fn new(code: i64, message: String, data: Option<Value>, context: Option<ErrorContext>) -> Self {
        Self {
            code,
            message,
            data,
            context,
        }
    }
    
    /// Create a parse error
    pub fn parse_error() -> Self {
        Self::new(-32700, "Parse error".to_string(), None, None)
    }
    
    /// Create an invalid request error
    pub fn invalid_request() -> Self {
        Self::new(-32600, "Invalid Request".to_string(), None, None)
    }
    
    /// Create a method not found error
    pub fn method_not_found(method: &str) -> Self {
        Self::new(-32601, format!("Method not found: {}", method), None, None)
    }
    
    /// Create an invalid params error
    pub fn invalid_params(method: &str, reason: &str) -> Self {
        Self::new(-32602, format!("Invalid parameters for {}: {}", method, reason), None, None)
    }
    
    /// Create an internal error
    pub fn internal_error(message: &str) -> Self {
        Self::new(-32603, format!("Internal error: {}", message), None, None)
    }
    
    /// Create a rate limit error
    pub fn rate_limit_error() -> Self {
        Self::new(-429, "Rate limit exceeded".to_string(), None, None)
    }
} 