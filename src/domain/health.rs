use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// Service is fully operational
    Healthy,
    /// Service is partially available (degraded)
    Degraded,
    /// Service is unavailable
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall health status
    pub status: HealthStatus,
    /// Detailed health information
    pub details: Value,
}

impl HealthResponse {
    /// Create a new health response
    pub fn new(status: HealthStatus, details: Value) -> Self {
        Self { status, details }
    }

    /// Check if the service is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy)
    }

    /// Check if the service is available (healthy or degraded)
    pub fn is_available(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Get HTTP status code for the health status
    pub fn http_status_code(&self) -> u16 {
        match self.status {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200, // Still return 200 for degraded
            HealthStatus::Unhealthy => 503,
        }
    }
}
