//! Common test utilities and mock implementations
//! 
//! This module provides shared utilities, mocks, and fixtures used across
//! all test modules.

use crate::{
    config::AppConfig,
    domain::rpc::{RpcRequest, RpcResponse, ClientInfo},
    infrastructure::http::models::{JsonRpcRequest, JsonRpcResponse, RequestContext},
    shared::error::AppResult,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Mock external RPC service for testing
pub struct MockExternalRpcService {
    responses: Arc<Mutex<std::collections::HashMap<String, RpcResponse>>>,
    call_count: Arc<Mutex<std::collections::HashMap<String, u32>>>,
}

impl MockExternalRpcService {
    /// Create a new mock service
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(std::collections::HashMap::new())),
            call_count: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Set a mock response for a method
    pub async fn set_response(&self, method: &str, response: RpcResponse) {
        let mut responses = self.responses.lock().await;
        responses.insert(method.to_string(), response);
    }

    /// Get call count for a method
    pub async fn get_call_count(&self, method: &str) -> u32 {
        let call_count = self.call_count.lock().await;
        *call_count.get(method).unwrap_or(&0)
    }

    /// Reset all call counts
    pub async fn reset_call_counts(&self) {
        let mut call_count = self.call_count.lock().await;
        call_count.clear();
    }

    /// Simulate processing a request
    pub async fn process_request(&self, request: &RpcRequest) -> AppResult<RpcResponse> {
        // Increment call count
        {
            let mut call_count = self.call_count.lock().await;
            *call_count.entry(request.method.clone()).or_insert(0) += 1;
        }

        // Return mock response or default
        let responses = self.responses.lock().await;
        if let Some(response) = responses.get(&request.method) {
            Ok(response.clone())
        } else {
            // Return default success response
            Ok(RpcResponse {
                result: Some(serde_json::json!({
                    "status": "success",
                    "method": request.method,
                    "timestamp": chrono::Utc::now().timestamp()
                })),
                error: None,
                id: request.id.clone(),
            })
        }
    }
}

impl Default for MockExternalRpcService {
    fn default() -> Self {
        Self::new()
    }
}

/// Test data fixtures
pub mod fixtures {
    use super::*;

    /// Create a test client info
    pub fn test_client_info() -> ClientInfo {
        ClientInfo {
            ip_address: "127.0.0.1".to_string(),
            user_agent: Some("test-client/1.0".to_string()),
            auth_token: None,
            request_id: Uuid::new_v4().to_string(),
        }
    }

    /// Create a test RPC request
    pub fn test_rpc_request(method: &str, params: Value) -> RpcRequest {
        RpcRequest {
            method: method.to_string(),
            parameters: Some(params),
            id: Some(serde_json::json!(1)),
            client_info: test_client_info(),
        }
    }

    /// Create a test JSON-RPC request
    pub fn test_json_rpc_request(method: &str, params: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(params),
            id: serde_json::json!(1),
        }
    }

    /// Create a test request context
    pub fn test_request_context() -> RequestContext {
        RequestContext::new(
            "127.0.0.1".to_string(),
            "getinfo".to_string(),
            Some(serde_json::json!([])),
        )
    }

    /// Create test block hash
    pub fn test_block_hash() -> String {
        "0000000000000000000000000000000000000000000000000000000000000000".to_string()
    }

    /// Create test transaction ID
    pub fn test_txid() -> String {
        "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()
    }

    /// Create test address
    pub fn test_address() -> String {
        "RTestAddress123456789012345678901234567890".to_string()
    }

    /// Create test currency ID
    pub fn test_currency_id() -> String {
        "iJhCezBExJHvtyH3fGhNnt2NhU4Ztkf2yq".to_string()
    }

    /// Create test identity
    pub fn test_identity() -> String {
        "test@".to_string()
    }

    /// Create sample getinfo response
    pub fn sample_getinfo_response() -> Value {
        serde_json::json!({
            "version": 1000000,
            "protocolversion": 170002,
            "VRSCversion": "1.0.0",
            "notarized": 1234567,
            "prevMoMheight": 1234566,
            "notarizedhash": "0000000000000000000000000000000000000000000000000000000000000000",
            "notarizedtxid": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "notarizedtxid_height": "1234567",
            "KMDnotarized_height": 1234567,
            "notarized_confirms": 1,
            "blocks": 1234567,
            "longestchain": 1234567,
            "timeoffset": 0,
            "tiptime": 1234567890,
            "connections": 8,
            "proxy": "",
            "difficulty": 1.0,
            "testnet": false,
            "paytxfee": 0.0001,
            "relayfee": 0.00001,
            "errors": "",
            "CCid": 1,
            "name": "VRSC",
            "p2pport": 27485,
            "rpcport": 27486,
            "magic": 1234567,
            "premine": 100000000,
            "eras": 1,
            "reward": "100000000",
            "halving": "210000",
            "decay": "0",
            "endsubsidy": "21000000",
            "veruspos": 1,
            "chainid": "0000000000000000000000000000000000000000000000000000000000000000"
        })
    }

    /// Create sample getblock response
    pub fn sample_getblock_response() -> Value {
        serde_json::json!({
            "hash": test_block_hash(),
            "confirmations": 1,
            "size": 1000,
            "height": 1234567,
            "version": 1,
            "merkleroot": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "tx": [
                "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            ],
            "time": 1234567890,
            "mediantime": 1234567890,
            "nonce": 123456789,
            "bits": "1d00ffff",
            "difficulty": 1.0,
            "chainwork": "0000000000000000000000000000000000000000000000000000000000000000",
            "previousblockhash": "0000000000000000000000000000000000000000000000000000000000000000",
            "nextblockhash": "0000000000000000000000000000000000000000000000000000000000000000"
        })
    }

    /// Create sample getrawtransaction response
    pub fn sample_getrawtransaction_response() -> Value {
        serde_json::json!({
            "txid": test_txid(),
            "hash": test_txid(),
            "version": 1,
            "size": 1000,
            "vsize": 1000,
            "weight": 4000,
            "locktime": 0,
            "vin": [
                {
                    "txid": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                    "vout": 0,
                    "sequence": 4294967295
                }
            ],
            "vout": [
                {
                    "value": 1.0,
                    "n": 0,
                    "scriptPubKey": {
                        "asm": "OP_DUP OP_HASH160 1234567890abcdef1234567890abcdef12345678 OP_EQUALVERIFY OP_CHECKSIG",
                        "hex": "76a9141234567890abcdef1234567890abcdef1234567888ac",
                        "reqSigs": 1,
                        "type": "pubkeyhash",
                        "addresses": [test_address()]
                    }
                }
            ],
            "blockhash": test_block_hash(),
            "confirmations": 1,
            "time": 1234567890,
            "blocktime": 1234567890
        })
    }

    /// Create sample getaddressbalance response
    pub fn sample_getaddressbalance_response() -> Value {
        serde_json::json!({
            "balance": 100.0,
            "received": 100.0,
            "currencybalance": {
                test_currency_id(): 50.0
            },
            "currencyreceived": {
                test_currency_id(): 50.0
            },
            "currencynames": {
                test_currency_id(): "TestCurrency"
            }
        })
    }

    /// Create sample getcurrency response
    pub fn sample_getcurrency_response() -> Value {
        serde_json::json!({
            "version": 1,
            "currencyid": test_currency_id(),
            "fullyqualifiedname": "TestCurrency",
            "currencyname": "TestCurrency",
            "parentid": "0000000000000000000000000000000000000000000000000000000000000000",
            "systemid": "0000000000000000000000000000000000000000000000000000000000000000",
            "notarizationprotocol": 1,
            "proofprotocol": 1,
            "startblock": 1234567,
            "endblock": 0,
            "currencies": [],
            "weights": [],
            "conversions": [],
            "minpreconversion": [],
            "preallocations": [],
            "initialcontributions": [],
            "idreferrals": [],
            "idimports": [],
            "options": 0,
            "definitiontxid": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "definitiontxout": 0,
            "launchsystemid": "0000000000000000000000000000000000000000000000000000000000000000",
            "launchcurrencyid": "0000000000000000000000000000000000000000000000000000000000000000",
            "launchnotarizationheight": 1234567,
            "launchnotarizationtxid": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "launchnotarizationtxout": 0,
            "launchsystemid": "0000000000000000000000000000000000000000000000000000000000000000",
            "launchcurrencyid": "0000000000000000000000000000000000000000000000000000000000000000",
            "launchnotarizationheight": 1234567,
            "launchnotarizationtxid": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "launchnotarizationtxout": 0
        })
    }

    /// Create sample getidentity response
    pub fn sample_getidentity_response() -> Value {
        serde_json::json!({
            "identity": {
                "version": 1,
                "flags": 0,
                "primaryaddresses": [test_address()],
                "minimumsignatures": 1,
                "privateaddress": "",
                "contentmap": {},
                "contentmultimap": {},
                "revocationauthority": test_identity(),
                "recoveryauthority": test_identity(),
                "timelock": 0
            },
            "status": "active",
            "canspendfor": true,
            "cansignfor": true,
            "blockheight": 1234567,
            "txid": test_txid(),
            "vout": 0
        })
    }
}

/// Test assertions and validators
pub mod assertions {
    use super::*;

    /// Assert that a response is a valid JSON-RPC response
    pub fn assert_valid_json_rpc_response(response: &JsonRpcResponse) {
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.id.is_some());
    }

    /// Assert that a response contains an error
    pub fn assert_json_rpc_error(response: &JsonRpcResponse, expected_code: i32) {
        assert_valid_json_rpc_response(response);
        assert!(response.error.is_some());
        let error = response.error.as_ref().unwrap();
        assert_eq!(error.code, expected_code);
    }

    /// Assert that a response contains a successful result
    pub fn assert_json_rpc_success(response: &JsonRpcResponse) {
        assert_valid_json_rpc_response(response);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    /// Assert that a value contains expected fields
    pub fn assert_contains_fields(value: &Value, expected_fields: &[&str]) {
        let obj = value.as_object().expect("Value should be an object");
        for field in expected_fields {
            assert!(obj.contains_key(*field), "Missing field: {}", field);
        }
    }

    /// Assert that a string is a valid hex string
    pub fn assert_valid_hex(hex: &str, expected_length: usize) {
        assert_eq!(hex.len(), expected_length);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Assert that a string is a valid address
    pub fn assert_valid_address(address: &str) {
        assert!(!address.is_empty());
        assert!(address.len() >= 26);
        assert!(address.len() <= 35);
    }
}

/// Performance testing utilities
pub mod performance {
    use std::time::{Duration, Instant};

    /// Measure execution time of a function
    pub async fn measure_time<F, Fut, T>(f: F) -> (T, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Assert that execution time is within acceptable bounds
    pub fn assert_performance(duration: Duration, max_duration: Duration) {
        assert!(
            duration <= max_duration,
            "Performance test failed: {:?} exceeded limit of {:?}",
            duration,
            max_duration
        );
    }

    /// Generate load for testing
    pub async fn generate_load<F, Fut, T>(
        concurrency: usize,
        iterations: usize,
        operation: F,
    ) -> Vec<(T, Duration)>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = T> + Send,
        T: Send,
    {
        use futures::stream::{self, StreamExt};
        use tokio::task::JoinSet;

        let mut results = Vec::new();
        let mut join_set = JoinSet::new();

        for _ in 0..iterations {
            for _ in 0..concurrency {
                let operation = &operation;
                join_set.spawn(async move {
                    let (result, duration) = measure_time(|| operation()).await;
                    (result, duration)
                });
            }

            while let Some(result) = join_set.join_next().await {
                results.push(result.unwrap());
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_external_rpc_service() {
        let mock_service = MockExternalRpcService::new();
        
        // Test default response
        let request = fixtures::test_rpc_request("getinfo", serde_json::json!([]));
        let response = mock_service.process_request(&request).await.unwrap();
        assert!(response.result.is_some());
        assert_eq!(mock_service.get_call_count("getinfo").await, 1);

        // Test custom response
        let custom_response = RpcResponse {
            result: Some(serde_json::json!({"custom": "data"})),
            error: None,
            id: Some(serde_json::json!(1)),
        };
        mock_service.set_response("getinfo", custom_response.clone()).await;
        
        let response = mock_service.process_request(&request).await.unwrap();
        assert_eq!(response.result, custom_response.result);
        assert_eq!(mock_service.get_call_count("getinfo").await, 2);
    }

    #[test]
    fn test_fixtures() {
        let client_info = fixtures::test_client_info();
        assert_eq!(client_info.ip_address, "127.0.0.1");
        assert!(client_info.user_agent.is_some());

        let request = fixtures::test_rpc_request("getinfo", serde_json::json!([]));
        assert_eq!(request.method, "getinfo");
        assert!(request.parameters.is_some());

        let json_request = fixtures::test_json_rpc_request("getinfo", serde_json::json!([]));
        assert_eq!(json_request.method, "getinfo");
        assert_eq!(json_request.jsonrpc, "2.0");
    }

    #[test]
    fn test_assertions() {
        let success_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"test": "data"})),
            error: None,
            id: Some(serde_json::json!(1)),
        };
        assertions::assert_json_rpc_success(&success_response);

        let error_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(crate::infrastructure::http::models::JsonRpcError::method_not_found()),
            id: Some(serde_json::json!(1)),
        };
        assertions::assert_json_rpc_error(&error_response, -32601);

        let value = serde_json::json!({"field1": "value1", "field2": "value2"});
        assertions::assert_contains_fields(&value, &["field1", "field2"]);

        assertions::assert_valid_hex("1234567890abcdef", 16);
        assertions::assert_valid_address("RTestAddress123456789012345678901234567890");
    }

    #[tokio::test]
    async fn test_performance_utilities() {
        let (result, duration) = performance::measure_time(|| async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "test"
        }).await;

        assert_eq!(result, "test");
        assert!(duration >= Duration::from_millis(10));
        performance::assert_performance(duration, Duration::from_millis(100));
    }
} 