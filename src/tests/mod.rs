//! Comprehensive test suite for Verus RPC Server
//! 
//! This module provides a complete testing framework covering:
//! - Unit tests for all components
//! - Integration tests for HTTP endpoints
//! - End-to-end tests for RPC functionality
//! - Performance and load tests
//! - Security and validation tests
//! - Mock and fixture utilities

pub mod common;
pub mod integration;
pub mod unit;
pub mod performance;
pub mod security;
pub mod fixtures;

/// Test configuration and utilities
pub mod config {
    use crate::config::AppConfig;
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// Initialize test environment
    pub fn init() {
        INIT.call_once(|| {
            // Initialize tracing for tests
            tracing_subscriber::fmt()
                .with_env_filter("debug")
                .with_test_writer()
                .init();
        });
    }

    /// Create test configuration
    pub fn test_config() -> AppConfig {
        let mut config = AppConfig::default();
        
        // Configure for testing
        config.server.port = 0; // Use random port
        config.server.bind_address = "127.0.0.1".parse().unwrap();
        config.security.development_mode = true;
        config.cache.enabled = false; // Disable cache for tests
        config.rate_limit.enabled = false; // Disable rate limiting for tests
        
        config
    }

    /// Create production-like test configuration
    pub fn production_test_config() -> AppConfig {
        let mut config = test_config();
        config.security.development_mode = false;
        config.cache.enabled = true;
        config.rate_limit.enabled = true;
        config
    }
}

/// Test result types
pub type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Test utilities and helpers
pub mod utils {
    use super::*;
    use serde_json::Value;
    use std::time::Duration;
    use tokio::time::sleep;

    /// Wait for a condition to be true
    pub async fn wait_for<F, Fut>(mut condition: F, timeout: Duration) -> bool
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if condition().await {
                return true;
            }
            sleep(Duration::from_millis(10)).await;
        }
        false
    }

    /// Create a test JSON-RPC request
    pub fn create_rpc_request(method: &str, params: Value, id: Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        })
    }

    /// Create a test JSON-RPC response
    pub fn create_rpc_response(result: Value, id: Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id
        })
    }

    /// Create a test JSON-RPC error response
    pub fn create_rpc_error(code: i32, message: &str, id: Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": code,
                "message": message
            },
            "id": id
        })
    }

    /// Assert JSON-RPC response structure
    pub fn assert_rpc_response(response: &Value) {
        assert!(response.is_object());
        let obj = response.as_object().unwrap();
        assert!(obj.contains_key("jsonrpc"));
        assert_eq!(obj["jsonrpc"], "2.0");
        assert!(obj.contains_key("id"));
    }

    /// Assert JSON-RPC error response structure
    pub fn assert_rpc_error(response: &Value, expected_code: i32) {
        assert_rpc_response(response);
        let obj = response.as_object().unwrap();
        assert!(obj.contains_key("error"));
        
        let error = &obj["error"];
        assert!(error.is_object());
        let error_obj = error.as_object().unwrap();
        assert!(error_obj.contains_key("code"));
        assert!(error_obj.contains_key("message"));
        assert_eq!(error_obj["code"], expected_code);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_initialization() {
        config::init();
        let test_config = config::test_config();
        assert!(test_config.security.development_mode);
        assert!(!test_config.cache.enabled);
        assert!(!test_config.rate_limit.enabled);
    }

    #[test]
    fn test_production_config() {
        let prod_config = config::production_test_config();
        assert!(!prod_config.security.development_mode);
        assert!(prod_config.cache.enabled);
        assert!(prod_config.rate_limit.enabled);
    }

    #[test]
    fn test_rpc_request_creation() {
        let request = utils::create_rpc_request(
            "getinfo",
            serde_json::json!([]),
            serde_json::json!(1)
        );
        
        assert_eq!(request["method"], "getinfo");
        assert_eq!(request["params"], serde_json::json!([]));
        assert_eq!(request["id"], 1);
        assert_eq!(request["jsonrpc"], "2.0");
    }

    #[test]
    fn test_rpc_response_creation() {
        let result = serde_json::json!({"version": "1.0.0"});
        let response = utils::create_rpc_response(result.clone(), serde_json::json!(1));
        
        assert_eq!(response["result"], result);
        assert_eq!(response["id"], 1);
        assert_eq!(response["jsonrpc"], "2.0");
    }

    #[test]
    fn test_rpc_error_creation() {
        let error = utils::create_rpc_error(-32601, "Method not found", serde_json::json!(1));
        
        assert_eq!(error["error"]["code"], -32601);
        assert_eq!(error["error"]["message"], "Method not found");
        assert_eq!(error["id"], 1);
        assert_eq!(error["jsonrpc"], "2.0");
    }

    #[test]
    fn test_response_assertions() {
        let response = utils::create_rpc_response(
            serde_json::json!({"test": "data"}),
            serde_json::json!(1)
        );
        utils::assert_rpc_response(&response);
        
        let error = utils::create_rpc_error(-32601, "Method not found", serde_json::json!(1));
        utils::assert_rpc_error(&error, -32601);
    }
} 