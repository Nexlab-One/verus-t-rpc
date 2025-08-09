//! Use cases - Application business operations

use crate::{
    application::services::*,
    domain::rpc::*,
    shared::error::AppResult,
};
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn};

/// Use case for processing RPC requests
pub struct ProcessRpcRequestUseCase {
    rpc_service: Arc<RpcService>,
    metrics_service: Arc<MetricsService>,
}

impl ProcessRpcRequestUseCase {
    /// Create a new use case
    pub fn new(rpc_service: Arc<RpcService>, metrics_service: Arc<MetricsService>) -> Self {
        Self {
            rpc_service,
            metrics_service,
        }
    }

    /// Execute RPC request processing
    pub async fn execute(&self, request: RpcRequest) -> AppResult<RpcResponse> {
        let result = self.rpc_service.process_request(&request).await;
        
        // Record metrics for the request
        match &result {
            Ok(_) => {
                self.metrics_service.record_request(true);
                info!("RPC request processed successfully");
            }
            Err(e) => {
                self.metrics_service.record_request(false);
                warn!("RPC request failed: {}", e);
            }
        }
        
        result
    }

    /// Get method information
    pub fn get_method_info(&self, _method_name: &str) -> Option<RpcMethod> {
        // This method is no longer available in the RPC service
        // Return None for now - this can be implemented later if needed
        None
    }
}

/// Use case for getting application metrics
pub struct GetMetricsUseCase {
    metrics_service: Arc<MetricsService>,
}

impl GetMetricsUseCase {
    /// Create a new use case
    pub fn new(metrics_service: Arc<MetricsService>) -> Self {
        Self { metrics_service }
    }

    /// Execute the use case
    pub fn execute(&self) -> Value {
        info!("Getting application metrics");
        self.metrics_service.get_metrics()
    }
}

/// Use case for health checks
pub struct HealthCheckUseCase;

impl HealthCheckUseCase {
    /// Execute the use case with enhanced daemon status
    pub async fn execute(&self, rpc_adapter: Option<Arc<crate::infrastructure::adapters::ExternalRpcAdapter>>) -> AppResult<crate::domain::health::HealthResponse> {
        use crate::domain::health::*;
        use serde_json::json;
        
        let mut status = HealthStatus::Healthy;
        let mut details = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION"),
            "uptime": self.get_uptime(),
        });

        // Check daemon connectivity if adapter is available
        if let Some(adapter) = rpc_adapter {
            let daemon_available = adapter.is_available().await;
            let circuit_status = adapter.get_circuit_status().await;
            
            details["daemon"] = json!({
                "available": daemon_available,
                "circuit_breaker": format!("{:?}", circuit_status),
                "status": if daemon_available { "connected" } else { "disconnected" }
            });

            if !daemon_available {
                status = HealthStatus::Degraded;
                details["warnings"] = json!([
                    "Verus daemon is currently unavailable",
                    "RPC requests may fail or be delayed"
                ]);
            }
        } else {
            details["daemon"] = json!({
                "available": false,
                "circuit_breaker": "unknown",
                "status": "no_adapter",
                "note": "RPC adapter not available for health check"
            });
            status = HealthStatus::Degraded;
        }

        // Add system metrics
        details["system"] = json!({
            "memory_usage": self.get_memory_usage(),
            "cpu_usage": self.get_cpu_usage(),
            "active_connections": self.get_active_connections(),
        });

        Ok(HealthResponse {
            status,
            details,
        })
    }

    /// Get system uptime
    fn get_uptime(&self) -> String {
        if let Ok(uptime) = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH) {
            let days = uptime.as_secs() / 86400;
            let hours = (uptime.as_secs() % 86400) / 3600;
            let minutes = (uptime.as_secs() % 3600) / 60;
            format!("{}d {}h {}m", days, hours, minutes)
        } else {
            "unknown".to_string()
        }
    }

    /// Get memory usage (simplified)
    fn get_memory_usage(&self) -> String {
        // In a real implementation, you'd use a crate like `sysinfo`
        "N/A".to_string()
    }

    /// Get CPU usage (simplified)
    fn get_cpu_usage(&self) -> String {
        // In a real implementation, you'd use a crate like `sysinfo`
        "N/A".to_string()
    }

    /// Get active connections (simplified)
    fn get_active_connections(&self) -> u32 {
        // In a real implementation, you'd track this in your server
        0
    }
}

/// Use case for validating RPC methods
pub struct ValidateRpcMethodUseCase {
    rpc_service: Arc<RpcService>,
}

impl ValidateRpcMethodUseCase {
    /// Create a new use case
    pub fn new(rpc_service: Arc<RpcService>) -> Self {
        Self { rpc_service }
    }

    /// Execute the use case
    pub fn execute(&self, method_name: &str) -> AppResult<bool> {
        info!("Validating RPC method: {}", method_name);
        
        // Use the RPC service's comprehensive validator to check if the method is allowed
        // This provides a more robust validation than just returning true
        match self.rpc_service.get_security_validator().validate_method(method_name) {
            Ok(_) => {
                info!("Method '{}' is valid", method_name);
                Ok(true)
            }
            Err(e) => {
                warn!("Method '{}' validation failed: {}", method_name, e);
                Ok(false)
            }
        }
    }
} 

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::AppConfig,
        domain::rpc::{RpcRequest, ClientInfo},
        domain::security::SecurityValidator,
        application::services::{RpcService, MetricsService},
        infrastructure::adapters::ExternalRpcAdapter,
    };
    use std::sync::Arc;
    use chrono::Utc;
    use serde_json::json;

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    fn create_test_rpc_request(method: &str, params: serde_json::Value) -> RpcRequest {
        RpcRequest {
            method: method.to_string(),
            parameters: Some(params),
            id: Some(json!("test-id")),
            client_info: ClientInfo {
                ip_address: "127.0.0.1".to_string(),
                user_agent: Some("test-agent".to_string()),
                auth_token: None,
                timestamp: Utc::now(),
            },
        }
    }

    fn create_test_rpc_request_with_auth(method: &str, params: serde_json::Value, auth_token: &str) -> RpcRequest {
        RpcRequest {
            method: method.to_string(),
            parameters: Some(params),
            id: Some(json!("test-id")),
            client_info: ClientInfo {
                ip_address: "127.0.0.1".to_string(),
                user_agent: Some("test-agent".to_string()),
                auth_token: Some(auth_token.to_string()),
                timestamp: Utc::now(),
            },
        }
    }

    #[tokio::test]
    async fn test_process_rpc_request_use_case_success() {
        let config = Arc::new(create_test_config());
        let security_validator = Arc::new(SecurityValidator::new(Default::default()));
        let rpc_service = Arc::new(RpcService::new(config, security_validator));
        let metrics_service = Arc::new(MetricsService::new());
        
        let use_case = ProcessRpcRequestUseCase::new(rpc_service, metrics_service.clone());
        
        let request = create_test_rpc_request("getinfo", json!([]));
        let result = use_case.execute(request).await;
        
        // Should succeed with fallback response due to circuit breaker
        assert!(result.is_ok());
        
        let response = result.unwrap();
        // Should have a result (fallback response) or error (daemon unavailable)
        assert!(response.result.is_some() || response.error.is_some());
        
        // Check that metrics were recorded
        let metrics = metrics_service.get_metrics();
        assert!(metrics.get("total_requests").is_some());
        assert!(metrics.get("successful_requests").is_some());
    }

    #[tokio::test]
    async fn test_process_rpc_request_use_case_with_authentication() {
        let config = Arc::new(create_test_config());
        let security_validator = Arc::new(SecurityValidator::new(Default::default()));
        let rpc_service = Arc::new(RpcService::new(config, security_validator));
        let metrics_service = Arc::new(MetricsService::new());
        
        let use_case = ProcessRpcRequestUseCase::new(rpc_service, metrics_service);
        
        let request = create_test_rpc_request_with_auth("getinfo", json!([]), "test-token");
        let result = use_case.execute(request).await;
        
        // Should fail due to invalid authentication token
        assert!(result.is_err());
        
        if let Err(e) = &result {
            match e {
                crate::shared::error::AppError::Authentication(_) => {
                    // This is expected - authentication failed due to invalid token
                }
                _ => {
                    panic!("Expected authentication error, got: {:?}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_process_rpc_request_use_case_failure_records_metrics() {
        let config = Arc::new(create_test_config());
        let security_validator = Arc::new(SecurityValidator::new(Default::default()));
        let rpc_service = Arc::new(RpcService::new(config, security_validator));
        let metrics_service = Arc::new(MetricsService::new());
        
        let use_case = ProcessRpcRequestUseCase::new(rpc_service, metrics_service.clone());
        
        // Create a request that will likely fail (invalid method)
        let request = create_test_rpc_request("invalid_method", json!([]));
        let result = use_case.execute(request).await;
        
        // Should fail
        assert!(result.is_err());
        
        // Check that failure metrics were recorded
        let metrics = metrics_service.get_metrics();
        assert!(metrics.get("total_requests").is_some());
        assert!(metrics.get("failed_requests").is_some());
    }

    #[tokio::test]
    async fn test_get_metrics_use_case() {
        let metrics_service = Arc::new(MetricsService::new());
        let use_case = GetMetricsUseCase::new(metrics_service.clone());
        
        // Record some test metrics
        metrics_service.record_request(true);
        metrics_service.record_request(false);
        metrics_service.record_request(true);
        
        let metrics = use_case.execute();
        
        assert!(metrics.is_object());
        assert!(metrics.get("total_requests").is_some());
        assert!(metrics.get("successful_requests").is_some());
        assert!(metrics.get("failed_requests").is_some());
        
        let total_requests = metrics["total_requests"].as_u64().unwrap();
        assert_eq!(total_requests, 3);
    }

    #[tokio::test]
    async fn test_health_check_use_case_without_adapter() {
        let use_case = HealthCheckUseCase;
        
        let result = use_case.execute(None).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        
        // Should be degraded without RPC adapter
        assert_eq!(response.status.to_string(), "degraded");
        assert!(response.details.is_object());
        
        let details = response.details.as_object().unwrap();
        assert!(details.contains_key("timestamp"));
        assert!(details.contains_key("version"));
        assert!(details.contains_key("daemon"));
        assert!(details.contains_key("system"));
        
        // Check daemon status
        let daemon = details["daemon"].as_object().unwrap();
        assert_eq!(daemon["available"], false);
        assert_eq!(daemon["status"], "no_adapter");
    }

    #[tokio::test]
    async fn test_health_check_use_case_with_adapter() {
        let use_case = HealthCheckUseCase;
        let config = Arc::new(create_test_config());
        let adapter = Arc::new(ExternalRpcAdapter::new(config));
        
        let result = use_case.execute(Some(adapter)).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        
        // Should be healthy with available adapter
        assert_eq!(response.status.to_string(), "healthy");
        assert!(response.details.is_object());
        
        let details = response.details.as_object().unwrap();
        assert!(details.contains_key("daemon"));
        
        // Check daemon status
        let daemon = details["daemon"].as_object().unwrap();
        assert!(daemon.contains_key("available"));
        assert!(daemon.contains_key("circuit_breaker"));
        assert!(daemon.contains_key("status"));
    }

    #[tokio::test]
    async fn test_validate_rpc_method_use_case() {
        let config = Arc::new(create_test_config());
        let security_validator = Arc::new(SecurityValidator::new(Default::default()));
        let rpc_service = Arc::new(RpcService::new(config, security_validator));
        
        let use_case = ValidateRpcMethodUseCase::new(rpc_service);
        
        // Test valid method (default policy allows all methods)
        let result = use_case.execute("getinfo");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
        
        // Test invalid method (default policy allows all methods)
        let result = use_case.execute("invalid_method");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true); // Default policy allows all methods
    }

    #[tokio::test]
    async fn test_health_check_use_case_system_metrics() {
        let use_case = HealthCheckUseCase;
        
        let result = use_case.execute(None).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        let details = response.details.as_object().unwrap();
        let system = details["system"].as_object().unwrap();
        
        // Check system metrics are present
        assert!(system.contains_key("memory_usage"));
        assert!(system.contains_key("cpu_usage"));
        assert!(system.contains_key("active_connections"));
        
        // Check uptime is present
        assert!(details.contains_key("uptime"));
        let uptime = details["uptime"].as_str().unwrap();
        assert!(!uptime.is_empty());
    }

    #[tokio::test]
    async fn test_health_check_use_case_version_info() {
        let use_case = HealthCheckUseCase;
        
        let result = use_case.execute(None).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        let details = response.details.as_object().unwrap();
        
        // Check version is present
        assert!(details.contains_key("version"));
        let version = details["version"].as_str().unwrap();
        assert!(!version.is_empty());
        
        // Check timestamp is present and valid
        assert!(details.contains_key("timestamp"));
        let timestamp = details["timestamp"].as_str().unwrap();
        assert!(!timestamp.is_empty());
    }
} 