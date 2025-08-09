use crate::{
    domain::health::*,
    infrastructure::adapters::ExternalRpcAdapter,
    shared::error::AppResult,
};
use std::sync::Arc;
use serde_json::json;

/// Health check use case
pub struct HealthCheckUseCase;

impl HealthCheckUseCase {
    /// Create a new health check use case
    pub fn new() -> Self {
        Self
    }

    /// Execute health check with enhanced daemon status
    pub async fn execute(&self, rpc_adapter: Option<Arc<ExternalRpcAdapter>>) -> AppResult<HealthResponse> {
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
