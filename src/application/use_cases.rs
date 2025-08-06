//! Use cases - Application business operations

use crate::{
    application::services::*,
    domain::rpc::*,
    shared::error::AppResult,
};
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

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

    /// Execute the use case
    pub async fn execute(&self, request: RpcRequest) -> AppResult<RpcResponse> {
        info!(
            method = %request.method,
            client_ip = %request.client_info.ip_address,
            "Processing RPC request use case"
        );

        // Process the request
        let result = self.rpc_service.process_request(request).await;

        // Record metrics
        self.metrics_service.record_request(result.is_ok());

        result
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
    /// Execute the use case
    pub fn execute(&self) -> Value {
        info!("Performing health check");
        serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now(),
            "version": env!("CARGO_PKG_VERSION"),
        })
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
        info!(method = %method_name, "Validating RPC method");
        
        if let Some(method_info) = self.rpc_service.get_method_info(method_name) {
            Ok(method_info.required_permissions.is_empty())
        } else {
            Ok(false)
        }
    }
} 