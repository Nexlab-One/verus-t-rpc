//! Integration tests for Verus RPC Server
//! 
//! This module provides comprehensive integration tests covering:
//! - HTTP server startup and shutdown
//! - RPC endpoint functionality
//! - End-to-end request processing
//! - Error handling and edge cases
//! - Performance under load

use crate::{
    config::AppConfig,
    infrastructure::http::server::HttpServer,
    tests::{
        common::{fixtures, assertions},
        config,
        TestResult,
    },
};
use serde_json::Value;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use warp::test::request;

/// Test server instance for integration tests
pub struct TestServer {
    pub server: HttpServer,
    pub addr: SocketAddr,
}

impl TestServer {
    /// Create a new test server instance
    pub async fn new() -> TestResult<Self> {
        config::init();
        let config = config::test_config();
        
        // Create server
        let server = HttpServer::new(config).await?;
        
        // Get a random available port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        drop(listener);
        
        Ok(Self { server, addr })
    }

    /// Start the test server
    pub async fn start(self) -> TestResult<RunningTestServer> {
        let routes = self.server.create_routes();
        let (addr, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(self.addr, async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for ctrl+c");
            });
        
        let handle = tokio::spawn(server);
        
        Ok(RunningTestServer {
            addr,
            handle,
        })
    }
}

/// Running test server instance
pub struct RunningTestServer {
    pub addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
}

impl RunningTestServer {
    /// Stop the test server
    pub async fn stop(self) {
        self.handle.abort();
        let _ = self.handle.await;
    }

    /// Get the server URL
    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

/// Integration test utilities
pub mod utils {
    use super::*;
    use warp::http::StatusCode;

    /// Send a JSON-RPC request to the test server
    pub async fn send_rpc_request(
        server_url: &str,
        method: &str,
        params: Value,
    ) -> TestResult<(StatusCode, Value)> {
        let request_body = fixtures::test_json_rpc_request(method, params);
        
        // Create a simple test configuration
        let mut config = crate::config::AppConfig::default();
        config.server.port = 0;
        config.server.bind_address = "127.0.0.1".parse().unwrap();
        config.security.development_mode = true;
        config.cache.enabled = false;
        config.rate_limit.enabled = false;
        
        let server = crate::infrastructure::http::server::HttpServer::new(config).await?;
        let routes = server.create_routes();
        
        let response = request()
            .method("POST")
            .header("Content-Type", "application/json")
            .header("x-forwarded-for", "127.0.0.1")
            .json(&request_body)
            .reply(&routes)
            .await;

        let status = response.status();
        let body: Value = serde_json::from_slice(response.body())?;
        
        Ok((status, body))
    }

    /// Send a health check request
    pub async fn send_health_request(server_url: &str) -> TestResult<(StatusCode, Value)> {
        // Create a simple test configuration
        let mut config = crate::config::AppConfig::default();
        config.server.port = 0;
        config.server.bind_address = "127.0.0.1".parse().unwrap();
        config.security.development_mode = true;
        config.cache.enabled = false;
        config.rate_limit.enabled = false;
        
        let server = crate::infrastructure::http::server::HttpServer::new(config).await?;
        let routes = server.create_routes();
        
        let response = request()
            .method("GET")
            .path("/health")
            .reply(&routes)
            .await;

        let status = response.status();
        let body: Value = serde_json::from_slice(response.body())?;
        
        Ok((status, body))
    }

    /// Send a metrics request
    pub async fn send_metrics_request(server_url: &str) -> TestResult<(StatusCode, Value)> {
        // Create a simple test configuration
        let mut config = crate::config::AppConfig::default();
        config.server.port = 0;
        config.server.bind_address = "127.0.0.1".parse().unwrap();
        config.security.development_mode = true;
        config.cache.enabled = false;
        config.rate_limit.enabled = false;
        
        let server = crate::infrastructure::http::server::HttpServer::new(config).await?;
        let routes = server.create_routes();
        
        let response = request()
            .method("GET")
            .path("/metrics")
            .reply(&routes)
            .await;

        let status = response.status();
        let body: Value = serde_json::from_slice(response.body())?;
        
        Ok((status, body))
    }

    /// Send a prometheus metrics request
    pub async fn send_prometheus_request(server_url: &str) -> TestResult<(StatusCode, String)> {
        // Create a simple test configuration
        let mut config = crate::config::AppConfig::default();
        config.server.port = 0;
        config.server.bind_address = "127.0.0.1".parse().unwrap();
        config.security.development_mode = true;
        config.cache.enabled = false;
        config.rate_limit.enabled = false;
        
        let server = crate::infrastructure::http::server::HttpServer::new(config).await?;
        let routes = server.create_routes();
        
        let response = request()
            .method("GET")
            .path("/prometheus")
            .reply(&routes)
            .await;

        let status = response.status();
        let body = String::from_utf8(response.body().to_vec())?;
        
        Ok((status, body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let test_server = TestServer::new().await.unwrap();
        assert_eq!(test_server.addr.ip().to_string(), "127.0.0.1");
        assert!(test_server.addr.port() > 0);
    }

    #[tokio::test]
    async fn test_getinfo_endpoint() {
        let (status, body) = utils::send_rpc_request(
            "http://127.0.0.1:8080",
            "getinfo",
            serde_json::json!([]),
        ).await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assertions::assert_rpc_response(&body);
        
        // Should have jsonrpc and id fields
        assert_eq!(body["jsonrpc"], "2.0");
        assert!(body.get("id").is_some());
    }

    #[tokio::test]
    async fn test_getblock_endpoint() {
        let block_hash = fixtures::test_block_hash();
        let (status, body) = utils::send_rpc_request(
            "http://127.0.0.1:8080",
            "getblock",
            serde_json::json!([block_hash, true]),
        ).await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assertions::assert_rpc_response(&body);
    }

    #[tokio::test]
    async fn test_getrawtransaction_endpoint() {
        let txid = fixtures::test_txid();
        let (status, body) = utils::send_rpc_request(
            "http://127.0.0.1:8080",
            "getrawtransaction",
            serde_json::json!([txid, 1]),
        ).await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assertions::assert_rpc_response(&body);
    }

    #[tokio::test]
    async fn test_getaddressbalance_endpoint() {
        let addresses = serde_json::json!({
            "addresses": [fixtures::test_address()]
        });
        let (status, body) = utils::send_rpc_request(
            "http://127.0.0.1:8080",
            "getaddressbalance",
            addresses,
        ).await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assertions::assert_rpc_response(&body);
    }

    #[tokio::test]
    async fn test_getcurrency_endpoint() {
        let currency_id = fixtures::test_currency_id();
        let (status, body) = utils::send_rpc_request(
            "http://127.0.0.1:8080",
            "getcurrency",
            serde_json::json!([currency_id]),
        ).await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assertions::assert_rpc_response(&body);
    }

    #[tokio::test]
    async fn test_getidentity_endpoint() {
        let identity = fixtures::test_identity();
        let (status, body) = utils::send_rpc_request(
            "http://127.0.0.1:8080",
            "getidentity",
            serde_json::json!([identity]),
        ).await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assertions::assert_rpc_response(&body);
    }

    #[tokio::test]
    async fn test_invalid_method() {
        let (status, body) = utils::send_rpc_request(
            "http://127.0.0.1:8080",
            "invalid_method",
            serde_json::json!([]),
        ).await.unwrap();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assertions::assert_rpc_error(&body, -32601);
    }

    #[tokio::test]
    async fn test_invalid_parameters() {
        let (status, body) = utils::send_rpc_request(
            "http://127.0.0.1:8080",
            "getblock",
            serde_json::json!(["invalid_hash"]), // Too short
        ).await.unwrap();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assertions::assert_rpc_error(&body, -32602);
    }

    #[tokio::test]
    async fn test_missing_parameters() {
        let (status, body) = utils::send_rpc_request(
            "http://127.0.0.1:8080",
            "getblock",
            serde_json::json!([]), // Missing required hash parameter
        ).await.unwrap();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assertions::assert_rpc_error(&body, -32602);
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let (status, body) = utils::send_health_request("http://127.0.0.1:8080").await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assert!(body.is_object());
        assert!(body.get("status").is_some());
        assert_eq!(body["status"], "healthy");
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let (status, body) = utils::send_metrics_request("http://127.0.0.1:8080").await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assert!(body.is_object());
    }

    #[tokio::test]
    async fn test_prometheus_endpoint() {
        let (status, body) = utils::send_prometheus_request("http://127.0.0.1:8080").await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("# HELP"));
        assert!(body.contains("# TYPE"));
    }

    #[tokio::test]
    async fn test_cors_headers() {
        // Create a simple test configuration
        let mut config = crate::config::AppConfig::default();
        config.server.port = 0;
        config.server.bind_address = "127.0.0.1".parse().unwrap();
        config.security.development_mode = true;
        config.cache.enabled = false;
        config.rate_limit.enabled = false;
        
        let server = crate::infrastructure::http::server::HttpServer::new(config).await.unwrap();
        let routes = server.create_routes();
        
        let response = request()
            .method("OPTIONS")
            .header("Origin", "http://localhost:3000")
            .header("Access-Control-Request-Method", "POST")
            .header("Access-Control-Request-Headers", "Content-Type")
            .reply(&routes)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("access-control-allow-origin"));
        assert!(response.headers().contains_key("access-control-allow-methods"));
        assert!(response.headers().contains_key("access-control-allow-headers"));
    }

    #[tokio::test]
    async fn test_content_type_validation() {
        // Create a simple test configuration
        let mut config = crate::config::AppConfig::default();
        config.server.port = 0;
        config.server.bind_address = "127.0.0.1".parse().unwrap();
        config.security.development_mode = true;
        config.cache.enabled = false;
        config.rate_limit.enabled = false;
        
        let server = crate::infrastructure::http::server::HttpServer::new(config).await.unwrap();
        let routes = server.create_routes();
        
        let response = request()
            .method("POST")
            .header("Content-Type", "text/plain") // Invalid content type
            .header("x-forwarded-for", "127.0.0.1")
            .body("invalid json")
            .reply(&routes)
            .await;

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_missing_x_forwarded_for() {
        let request_body = fixtures::test_json_rpc_request("getinfo", serde_json::json!([]));
        
        // Create a simple test configuration
        let mut config = crate::config::AppConfig::default();
        config.server.port = 0;
        config.server.bind_address = "127.0.0.1".parse().unwrap();
        config.security.development_mode = true;
        config.cache.enabled = false;
        config.rate_limit.enabled = false;
        
        let server = crate::infrastructure::http::server::HttpServer::new(config).await.unwrap();
        let routes = server.create_routes();
        
        let response = request()
            .method("POST")
            .header("Content-Type", "application/json")
            // Missing x-forwarded-for header
            .json(&request_body)
            .reply(&routes)
            .await;

        // Should still work in development mode
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_large_request_body() {
        let large_params = serde_json::json!({
            "data": "x".repeat(1024 * 1024) // 1MB of data
        });
        
        let (status, body) = utils::send_rpc_request(
            "http://127.0.0.1:8080",
            "getinfo",
            large_params,
        ).await.unwrap();

        // Should be rejected due to size limit
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        use futures::stream::{self, StreamExt};
        use tokio::task::JoinSet;

        let mut join_set = JoinSet::new();
        let num_requests = 10;

        for i in 0..num_requests {
            join_set.spawn(async move {
                let (status, body) = utils::send_rpc_request(
                    "http://127.0.0.1:8080",
                    "getinfo",
                    serde_json::json!([]),
                ).await.unwrap();

                (i, status, body)
            });
        }

        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            results.push(result.unwrap());
        }

        assert_eq!(results.len(), num_requests);
        for (_, status, body) in results {
            assert_eq!(status, StatusCode::OK);
            assertions::assert_rpc_response(&body);
        }
    }

    #[tokio::test]
    async fn test_method_parameter_validation() {
        // Test various parameter combinations for different methods
        let test_cases = vec![
            ("getblock", serde_json::json!([fixtures::test_block_hash(), true])),
            ("getrawtransaction", serde_json::json!([fixtures::test_txid(), 1])),
            ("getaddressbalance", serde_json::json!({
                "addresses": [fixtures::test_address()]
            })),
            ("getcurrency", serde_json::json!([fixtures::test_currency_id()])),
            ("getidentity", serde_json::json!([fixtures::test_identity()])),
        ];

        for (method, params) in test_cases {
            let (status, body) = utils::send_rpc_request(
                "http://127.0.0.1:8080",
                method,
                params,
            ).await.unwrap();

            assert_eq!(status, StatusCode::OK, "Method {} failed", method);
            assertions::assert_rpc_response(&body);
        }
    }

    #[tokio::test]
    async fn test_error_handling_edge_cases() {
        // Create a simple test configuration
        let mut config = crate::config::AppConfig::default();
        config.server.port = 0;
        config.server.bind_address = "127.0.0.1".parse().unwrap();
        config.security.development_mode = true;
        config.cache.enabled = false;
        config.rate_limit.enabled = false;
        
        let server = crate::infrastructure::http::server::HttpServer::new(config).await.unwrap();
        let routes = server.create_routes();
        
        // Test malformed JSON
        let response = request()
            .method("POST")
            .header("Content-Type", "application/json")
            .header("x-forwarded-for", "127.0.0.1")
            .body("{ invalid json }")
            .reply(&routes)
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Test missing method
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "params": [],
            "id": 1
        });
        
        let response = request()
            .method("POST")
            .header("Content-Type", "application/json")
            .header("x-forwarded-for", "127.0.0.1")
            .json(&request_body)
            .reply(&routes)
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Test invalid JSON-RPC version
        let request_body = serde_json::json!({
            "jsonrpc": "1.0",
            "method": "getinfo",
            "params": [],
            "id": 1
        });
        
        let response = request()
            .method("POST")
            .header("Content-Type", "application/json")
            .header("x-forwarded-for", "127.0.0.1")
            .json(&request_body)
            .reply(&routes)
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
} 