//! Test fixtures and mock data for Verus RPC Server tests
//! 
//! This module provides comprehensive test fixtures including:
//! - Mock RPC responses
//! - Test data generators
//! - Mock external services
//! - Test configuration builders

use crate::{
    config::AppConfig,
    domain::rpc::{RpcRequest, RpcResponse, ClientInfo},
    infrastructure::http::models::{JsonRpcRequest, JsonRpcResponse, RequestContext, JsonRpcError},
    tests::common::fixtures as common_fixtures,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Test data generator
pub struct TestDataGenerator {
    counter: Arc<Mutex<u64>>,
}

impl TestDataGenerator {
    /// Create a new test data generator
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Generate a unique test ID
    pub async fn generate_id(&self) -> u64 {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        *counter
    }

    /// Generate a test block hash
    pub fn generate_block_hash(&self) -> String {
        format!("{:064x}", rand::random::<u64>())
    }

    /// Generate a test transaction ID
    pub fn generate_txid(&self) -> String {
        format!("{:064x}", rand::random::<u64>())
    }

    /// Generate a test address
    pub fn generate_address(&self) -> String {
        format!("RTestAddress{:032x}", rand::random::<u64>())
    }

    /// Generate a test currency ID
    pub fn generate_currency_id(&self) -> String {
        format!("iJhCezBExJHvtyH3fGhNnt2NhU4Ztkf2yq{:016x}", rand::random::<u64>())
    }

    /// Generate a test identity
    pub fn generate_identity(&self) -> String {
        format!("test{:016x}@", rand::random::<u64>())
    }
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock external RPC service for testing
pub struct MockExternalRpcService {
    responses: Arc<Mutex<std::collections::HashMap<String, RpcResponse>>>,
    call_count: Arc<Mutex<std::collections::HashMap<String, u32>>>,
    delay_ms: u64,
}

impl MockExternalRpcService {
    /// Create a new mock external RPC service
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(std::collections::HashMap::new())),
            call_count: Arc::new(Mutex::new(std::collections::HashMap::new())),
            delay_ms: 0,
        }
    }

    /// Create a new mock service with delay
    pub fn with_delay(delay_ms: u64) -> Self {
        Self {
            responses: Arc::new(Mutex::new(std::collections::HashMap::new())),
            call_count: Arc::new(Mutex::new(std::collections::HashMap::new())),
            delay_ms,
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
    pub async fn process_request(&self, request: &RpcRequest) -> crate::shared::error::AppResult<RpcResponse> {
        // Simulate delay if configured
        if self.delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        }

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

/// Test configuration builder
pub struct TestConfigBuilder {
    config: AppConfig,
}

impl TestConfigBuilder {
    /// Create a new test configuration builder
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    /// Set server configuration
    pub fn with_server(mut self, host: &str, port: u16) -> Self {
        self.config.server.bind_address = host.parse().unwrap();
        self.config.server.port = port;
        self
    }

    /// Set development mode
    pub fn with_development_mode(mut self, enabled: bool) -> Self {
        self.config.security.development_mode = enabled;
        self
    }

    /// Set cache configuration
    pub fn with_cache(mut self, enabled: bool, redis_url: &str) -> Self {
        self.config.cache.enabled = enabled;
        self.config.cache.redis_url = redis_url.to_string();
        self
    }

    /// Set rate limiting configuration
    pub fn with_rate_limit(mut self, enabled: bool, requests_per_minute: u32) -> Self {
        self.config.rate_limit.enabled = enabled;
        self.config.rate_limit.requests_per_minute = requests_per_minute;
        self
    }

    /// Set security configuration
    pub fn with_security(mut self, enable_security_headers: bool, enable_request_logging: bool) -> Self {
        self.config.security.enable_security_headers = enable_security_headers;
        self.config.security.enable_request_logging = enable_request_logging;
        self
    }

    /// Build the configuration
    pub fn build(self) -> AppConfig {
        self.config
    }
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock response generators
pub mod responses {
    use super::*;

    /// Generate a mock getinfo response
    pub fn getinfo_response() -> Value {
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

    /// Generate a mock getblock response
    pub fn getblock_response(block_hash: &str) -> Value {
        serde_json::json!({
            "hash": block_hash,
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

    /// Generate a mock getrawtransaction response
    pub fn getrawtransaction_response(txid: &str) -> Value {
        serde_json::json!({
            "txid": txid,
            "hash": txid,
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
                        "addresses": [common_fixtures::test_address()]
                    }
                }
            ],
            "blockhash": common_fixtures::test_block_hash(),
            "confirmations": 1,
            "time": 1234567890,
            "blocktime": 1234567890
        })
    }

    /// Generate a mock getaddressbalance response
    pub fn getaddressbalance_response(addresses: &[String]) -> Value {
        let mut currency_balance = serde_json::Map::new();
        let mut currency_received = serde_json::Map::new();
        let mut currency_names = serde_json::Map::new();
        
        currency_balance.insert(common_fixtures::test_currency_id(), serde_json::json!(50.0));
        currency_received.insert(common_fixtures::test_currency_id(), serde_json::json!(50.0));
        currency_names.insert(common_fixtures::test_currency_id(), serde_json::json!("TestCurrency"));

        serde_json::json!({
            "balance": 100.0,
            "received": 100.0,
            "currencybalance": currency_balance,
            "currencyreceived": currency_received,
            "currencynames": currency_names
        })
    }

    /// Generate a mock getcurrency response
    pub fn getcurrency_response(currency_id: &str) -> Value {
        serde_json::json!({
            "version": 1,
            "currencyid": currency_id,
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
            "launchnotarizationtxout": 0
        })
    }

    /// Generate a mock getidentity response
    pub fn getidentity_response(identity: &str) -> Value {
        serde_json::json!({
            "identity": {
                "version": 1,
                "flags": 0,
                "primaryaddresses": [common_fixtures::test_address()],
                "minimumsignatures": 1,
                "privateaddress": "",
                "contentmap": {},
                "contentmultimap": {},
                "revocationauthority": identity,
                "recoveryauthority": identity,
                "timelock": 0
            },
            "status": "active",
            "canspendfor": true,
            "cansignfor": true,
            "blockheight": 1234567,
            "txid": common_fixtures::test_txid(),
            "vout": 0
        })
    }

    /// Generate a mock error response
    pub fn error_response(code: i32, message: &str) -> JsonRpcError {
        JsonRpcError {
            code,
            message: message.to_string(),
        }
    }

    /// Generate a mock success response
    pub fn success_response(result: Value, id: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: Some(id),
        }
    }
}

/// Mock request generators
pub mod requests {
    use super::*;

    /// Generate a mock getinfo request
    pub fn getinfo_request(id: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "getinfo".to_string(),
            params: Some(serde_json::json!([])),
            id,
        }
    }

    /// Generate a mock getblock request
    pub fn getblock_request(block_hash: &str, verbose: bool, id: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "getblock".to_string(),
            params: Some(serde_json::json!([block_hash, verbose])),
            id,
        }
    }

    /// Generate a mock getrawtransaction request
    pub fn getrawtransaction_request(txid: &str, verbose: i32, id: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "getrawtransaction".to_string(),
            params: Some(serde_json::json!([txid, verbose])),
            id,
        }
    }

    /// Generate a mock getaddressbalance request
    pub fn getaddressbalance_request(addresses: &[String], id: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "getaddressbalance".to_string(),
            params: Some(serde_json::json!({
                "addresses": addresses
            })),
            id,
        }
    }

    /// Generate a mock getcurrency request
    pub fn getcurrency_request(currency_id: &str, id: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "getcurrency".to_string(),
            params: Some(serde_json::json!([currency_id])),
            id,
        }
    }

    /// Generate a mock getidentity request
    pub fn getidentity_request(identity: &str, id: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "getidentity".to_string(),
            params: Some(serde_json::json!([identity])),
            id,
        }
    }

    /// Generate a mock invalid request
    pub fn invalid_request(id: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "1.0".to_string(), // Invalid version
            method: "invalid_method".to_string(),
            params: Some(serde_json::json!([])),
            id,
        }
    }
}

/// Test scenario builders
pub mod scenarios {
    use super::*;

    /// Test scenario for basic functionality
    pub struct BasicScenario {
        pub generator: TestDataGenerator,
        pub mock_service: MockExternalRpcService,
    }

    impl BasicScenario {
        /// Create a new basic scenario
        pub async fn new() -> Self {
            let generator = TestDataGenerator::new();
            let mock_service = MockExternalRpcService::new();
            
            // Set up default responses
            mock_service.set_response("getinfo", RpcResponse {
                result: Some(responses::getinfo_response()),
                error: None,
                id: Some(serde_json::json!(1)),
            }).await;

            Self {
                generator,
                mock_service,
            }
        }

        /// Generate a complete test scenario
        pub async fn generate_scenario(&self) -> (JsonRpcRequest, JsonRpcResponse) {
            let id = self.generator.generate_id().await;
            let request = requests::getinfo_request(serde_json::json!(id));
            let response = responses::success_response(
                responses::getinfo_response(),
                serde_json::json!(id)
            );
            
            (request, response)
        }
    }

    /// Test scenario for error conditions
    pub struct ErrorScenario {
        pub generator: TestDataGenerator,
    }

    impl ErrorScenario {
        /// Create a new error scenario
        pub fn new() -> Self {
            Self {
                generator: TestDataGenerator::new(),
            }
        }

        /// Generate an invalid method scenario
        pub async fn generate_invalid_method(&self) -> (JsonRpcRequest, JsonRpcResponse) {
            let id = self.generator.generate_id().await;
            let request = requests::invalid_request(serde_json::json!(id));
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(responses::error_response(-32601, "Method not found")),
                id: Some(serde_json::json!(id)),
            };
            
            (request, response)
        }

        /// Generate an invalid parameters scenario
        pub async fn generate_invalid_params(&self) -> (JsonRpcRequest, JsonRpcResponse) {
            let id = self.generator.generate_id().await;
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "getblock".to_string(),
                params: Some(serde_json::json!(["invalid_hash"])), // Too short
                id: serde_json::json!(id),
            };
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(responses::error_response(-32602, "Invalid params")),
                id: Some(serde_json::json!(id)),
            };
            
            (request, response)
        }
    }

    /// Test scenario for performance testing
    pub struct PerformanceScenario {
        pub generator: TestDataGenerator,
        pub mock_service: MockExternalRpcService,
    }

    impl PerformanceScenario {
        /// Create a new performance scenario
        pub async fn new(delay_ms: u64) -> Self {
            let generator = TestDataGenerator::new();
            let mock_service = MockExternalRpcService::with_delay(delay_ms);
            
            // Set up responses for performance testing
            mock_service.set_response("getinfo", RpcResponse {
                result: Some(responses::getinfo_response()),
                error: None,
                id: Some(serde_json::json!(1)),
            }).await;

            Self {
                generator,
                mock_service,
            }
        }

        /// Generate a performance test scenario
        pub async fn generate_scenario(&self) -> (JsonRpcRequest, JsonRpcResponse) {
            let id = self.generator.generate_id().await;
            let request = requests::getinfo_request(serde_json::json!(id));
            let response = responses::success_response(
                responses::getinfo_response(),
                serde_json::json!(id)
            );
            
            (request, response)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_data_generator() {
        let generator = TestDataGenerator::new();
        
        let block_hash = generator.generate_block_hash();
        assert_eq!(block_hash.len(), 64);
        assert!(block_hash.chars().all(|c| c.is_ascii_hexdigit()));
        
        let txid = generator.generate_txid();
        assert_eq!(txid.len(), 64);
        assert!(txid.chars().all(|c| c.is_ascii_hexdigit()));
        
        let address = generator.generate_address();
        assert!(address.starts_with("RTestAddress"));
        
        let currency_id = generator.generate_currency_id();
        assert!(currency_id.starts_with("iJhCezBExJHvtyH3fGhNnt2NhU4Ztkf2yq"));
        
        let identity = generator.generate_identity();
        assert!(identity.ends_with("@"));
    }

    #[tokio::test]
    async fn test_mock_external_rpc_service() {
        let mock_service = MockExternalRpcService::new();
        
        // Test default response
        let request = common_fixtures::test_rpc_request("getinfo", serde_json::json!([]));
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
    fn test_test_config_builder() {
        let config = TestConfigBuilder::new()
            .with_server("127.0.0.1", 8080)
            .with_development_mode(true)
            .with_cache(true, "redis://localhost:6379")
            .with_rate_limit(true, 100)
            .with_security(true, false)
            .build();

        assert_eq!(config.server.bind_address.to_string(), "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert!(config.security.development_mode);
        assert!(config.cache.enabled);
        assert!(config.rate_limit.enabled);
        assert_eq!(config.rate_limit.requests_per_minute, 100);
        assert!(config.security.enable_security_headers);
        assert!(!config.security.enable_request_logging);
    }

    #[test]
    fn test_response_generators() {
        let getinfo = responses::getinfo_response();
        assert!(getinfo.get("version").is_some());
        assert!(getinfo.get("blocks").is_some());
        
        let block_hash = "test_hash";
        let getblock = responses::getblock_response(block_hash);
        assert_eq!(getblock["hash"], block_hash);
        
        let txid = "test_txid";
        let getrawtx = responses::getrawtransaction_response(txid);
        assert_eq!(getrawtx["txid"], txid);
        
        let addresses = vec!["address1".to_string(), "address2".to_string()];
        let balance = responses::getaddressbalance_response(&addresses);
        assert!(balance.get("balance").is_some());
        
        let currency_id = "test_currency";
        let currency = responses::getcurrency_response(currency_id);
        assert_eq!(currency["currencyid"], currency_id);
        
        let identity = "test@";
        let identity_response = responses::getidentity_response(identity);
        assert_eq!(identity_response["identity"]["revocationauthority"], identity);
    }

    #[test]
    fn test_request_generators() {
        let id = serde_json::json!(1);
        
        let getinfo = requests::getinfo_request(id.clone());
        assert_eq!(getinfo.method, "getinfo");
        assert_eq!(getinfo.jsonrpc, "2.0");
        
        let getblock = requests::getblock_request("test_hash", true, id.clone());
        assert_eq!(getblock.method, "getblock");
        
        let getrawtx = requests::getrawtransaction_request("test_txid", 1, id.clone());
        assert_eq!(getrawtx.method, "getrawtransaction");
        
        let addresses = vec!["address1".to_string()];
        let balance = requests::getaddressbalance_request(&addresses, id.clone());
        assert_eq!(balance.method, "getaddressbalance");
        
        let currency = requests::getcurrency_request("test_currency", id.clone());
        assert_eq!(currency.method, "getcurrency");
        
        let identity = requests::getidentity_request("test@", id);
        assert_eq!(identity.method, "getidentity");
    }

    #[tokio::test]
    async fn test_scenarios() {
        // Test basic scenario
        let basic = BasicScenario::new().await;
        let (request, response) = basic.generate_scenario().await;
        assert_eq!(request.method, "getinfo");
        assert!(response.result.is_some());
        
        // Test error scenario
        let error = ErrorScenario::new();
        let (request, response) = error.generate_invalid_method().await;
        assert_eq!(request.jsonrpc, "1.0"); // Invalid version
        assert!(response.error.is_some());
        
        let (request, response) = error.generate_invalid_params().await;
        assert_eq!(request.method, "getblock");
        assert!(response.error.is_some());
        
        // Test performance scenario
        let performance = PerformanceScenario::new(10).await;
        let (request, response) = performance.generate_scenario().await;
        assert_eq!(request.method, "getinfo");
        assert!(response.result.is_some());
    }
} 