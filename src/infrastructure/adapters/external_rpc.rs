//! External RPC adapter for communicating with Verus daemon
//! 
//! This adapter handles HTTP communication with the external Verus daemon,
//! providing forward compatibility for future Rust daemon integration.

use crate::{
    domain::rpc::*,
    shared::error::AppResult,
    config::AppConfig,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,     // Normal operation
    Open,       // Circuit is open, failing fast
    HalfOpen,   // Testing if service is back
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
struct CircuitBreakerConfig {
    failure_threshold: u32,
    recovery_timeout: Duration,
    half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_requests: 3,
        }
    }
}

/// Circuit breaker state
#[derive(Debug)]
struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_count: AtomicU64,
    last_failure_time: RwLock<Option<Instant>>,
    half_open_requests: AtomicU64,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU64::new(0),
            last_failure_time: RwLock::new(None),
            half_open_requests: AtomicU64::new(0),
            config,
        }
    }

    async fn should_allow_request(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let last_failure = self.last_failure_time.read().await;
                if let Some(time) = *last_failure {
                    if time.elapsed() >= self.config.recovery_timeout {
                        // Try to transition to half-open
                        drop(state);
                        self.transition_to_half_open().await;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => {
                let current_requests = self.half_open_requests.load(Ordering::Relaxed);
                current_requests < self.config.half_open_max_requests as u64
            }
        }
    }

    async fn record_success(&self) {
        let mut state = self.state.write().await;
        match *state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                // Transition back to closed
                *state = CircuitState::Closed;
                self.failure_count.store(0, Ordering::Relaxed);
                self.half_open_requests.store(0, Ordering::Relaxed);
                info!("Circuit breaker: Transitioned to CLOSED (service recovered)");
            }
            CircuitState::Open => {
                // This shouldn't happen, but handle gracefully
                warn!("Circuit breaker: Unexpected success in OPEN state");
            }
        }
    }

    async fn record_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        let mut last_failure = self.last_failure_time.write().await;
        *last_failure = Some(Instant::now());

        let mut state = self.state.write().await;
        match *state {
            CircuitState::Closed => {
                if failure_count >= self.config.failure_threshold as u64 {
                    *state = CircuitState::Open;
                    warn!("Circuit breaker: Transitioned to OPEN (service failing)");
                }
            }
            CircuitState::HalfOpen => {
                *state = CircuitState::Open;
                self.half_open_requests.store(0, Ordering::Relaxed);
                warn!("Circuit breaker: Transitioned back to OPEN (service still failing)");
            }
            CircuitState::Open => {
                // Already open, just update failure time
            }
        }
    }

    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        if *state == CircuitState::Open {
            *state = CircuitState::HalfOpen;
            self.half_open_requests.store(0, Ordering::Relaxed);
            info!("Circuit breaker: Transitioned to HALF_OPEN (testing service)");
        }
    }

    async fn increment_half_open_requests(&self) {
        if *self.state.read().await == CircuitState::HalfOpen {
            self.half_open_requests.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Adapter for external RPC services with circuit breaker
pub struct ExternalRpcAdapter {
    _config: Arc<AppConfig>,
    circuit_breaker: Arc<CircuitBreaker>,
    daemon_available: AtomicBool,
}

impl ExternalRpcAdapter {
    /// Create a new external RPC adapter
    pub fn new(config: Arc<AppConfig>) -> Self {
        let circuit_config = config.verus.circuit_breaker
            .as_ref()
            .map(|cb_config| CircuitBreakerConfig {
                failure_threshold: cb_config.failure_threshold,
                recovery_timeout: Duration::from_secs(cb_config.recovery_timeout_seconds),
                half_open_max_requests: cb_config.half_open_max_requests,
            })
            .unwrap_or_else(CircuitBreakerConfig::default);
        
        Self {
            _config: config,
            circuit_breaker: Arc::new(CircuitBreaker::new(circuit_config)),
            daemon_available: AtomicBool::new(true),
        }
    }

    /// Send request to external RPC service with circuit breaker protection
    pub async fn send_request(&self, request: &RpcRequest) -> AppResult<RpcResponse> {
        // Check circuit breaker first
        if !self.circuit_breaker.should_allow_request().await {
            return Err(crate::shared::error::AppError::Rpc(
                "Service temporarily unavailable (circuit breaker open)".to_string()
            ));
        }

        // Increment half-open request counter if needed
        self.circuit_breaker.increment_half_open_requests().await;

        use reqwest::Client;
        use serde_json::json;
        use std::time::Duration;
        
        info!(
            method = %request.method,
            client_ip = %request.client_info.ip_address,
            "Sending request to external RPC service"
        );

        // Create HTTP client with timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(self._config.verus.timeout_seconds))
            .build()
            .map_err(|e| crate::shared::error::AppError::Config(format!("Failed to create HTTP client: {}", e)))?;

        // Create JSON-RPC request payload
        let payload = json!({
            "jsonrpc": "2.0",
            "method": request.method,
            "params": request.parameters,
            "id": request.id
        });

        // Send request with retries
        let mut last_error = None;
        for attempt in 0..=self._config.verus.max_retries {
            match client
                .post(&self._config.verus.rpc_url)
                .header("Content-Type", "application/json")
                .basic_auth(&self._config.verus.rpc_user, Some(&self._config.verus.rpc_password))
                .json(&payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json_response) => {
                                if let Some(result) = json_response.get("result") {
                                    // Record success
                                    self.circuit_breaker.record_success().await;
                                    self.daemon_available.store(true, Ordering::Relaxed);
                                    return Ok(RpcResponse::success(result.clone(), request.id.clone()));
                                } else if let Some(error) = json_response.get("error") {
                                    let error_msg = format!("RPC error: {}", error);
                                    self.circuit_breaker.record_failure().await;
                                    return Err(crate::shared::error::AppError::Rpc(error_msg));
                                } else {
                                    let error_msg = "Invalid RPC response".to_string();
                                    self.circuit_breaker.record_failure().await;
                                    return Err(crate::shared::error::AppError::Rpc(error_msg));
                                }
                            }
                            Err(e) => {
                                last_error = Some(format!("Failed to parse response: {}", e));
                                self.circuit_breaker.record_failure().await;
                            }
                        }
                    } else {
                        last_error = Some(format!("HTTP error: {}", response.status()));
                        self.circuit_breaker.record_failure().await;
                    }
                }
                Err(e) => {
                    last_error = Some(format!("Request failed: {}", e));
                    self.circuit_breaker.record_failure().await;
                }
            }
            
            if attempt < self._config.verus.max_retries {
                info!("RPC request failed, retrying... (attempt {}/{})", attempt + 1, self._config.verus.max_retries + 1);
                tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
            }
        }

        // Mark daemon as unavailable after all retries failed
        self.daemon_available.store(false, Ordering::Relaxed);
        Err(crate::shared::error::AppError::Rpc(format!("RPC request failed after {} attempts: {:?}", self._config.verus.max_retries + 1, last_error)))
    }

    /// Check if external service is available
    pub async fn is_available(&self) -> bool {
        self.daemon_available.load(Ordering::Relaxed) && 
        self.circuit_breaker.should_allow_request().await
    }

    /// Get circuit breaker status for monitoring
    pub async fn get_circuit_status(&self) -> CircuitState {
        self.circuit_breaker.state.read().await.clone()
    }

    /// Force reset circuit breaker (for admin operations)
    pub async fn reset_circuit_breaker(&self) {
        let mut state = self.circuit_breaker.state.write().await;
        *state = CircuitState::Closed;
        self.circuit_breaker.failure_count.store(0, Ordering::Relaxed);
        self.circuit_breaker.half_open_requests.store(0, Ordering::Relaxed);
        self.daemon_available.store(true, Ordering::Relaxed);
        info!("Circuit breaker: Manually reset to CLOSED");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::rpc::{RpcRequest, ClientInfo};
    use crate::config::AppConfig;

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    fn create_test_request() -> RpcRequest {
        RpcRequest {
            method: "getinfo".to_string(),
            parameters: Some(serde_json::Value::Array(vec![])),
            id: Some(serde_json::Value::String("test".to_string())),
            client_info: ClientInfo {
                ip_address: "127.0.0.1".to_string(),
                user_agent: Some("test-agent".to_string()),
                auth_token: None,
                timestamp: chrono::Utc::now(),
            },
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_initial_state() {
        let config = Arc::new(create_test_config());
        let adapter = ExternalRpcAdapter::new(config);
        
        let status = adapter.get_circuit_status().await;
        assert_eq!(status, CircuitState::Closed);
        assert!(adapter.is_available().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure_threshold() {
        let config = Arc::new(create_test_config());
        let adapter = ExternalRpcAdapter::new(config);
        let request = create_test_request();
        
        // Test that circuit breaker opens after failures
        let mut circuit_opened = false;
        for _ in 0..10 {
            let result = adapter.send_request(&request).await;
            assert!(result.is_err());
            
            let status = adapter.get_circuit_status().await;
            if status == CircuitState::Open {
                circuit_opened = true;
                break;
            }
        }
        
        // Circuit breaker should have opened at some point
        assert!(circuit_opened, "Circuit breaker should have opened after multiple failures");
        
        // After opening, requests should fail fast
        let result = adapter.send_request(&request).await;
        assert!(result.is_err());
        assert!(!adapter.is_available().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = Arc::new(create_test_config());
        let adapter = ExternalRpcAdapter::new(config);
        let request = create_test_request();
        
        // Open the circuit breaker
        let mut circuit_opened = false;
        for _ in 0..10 {
            let _ = adapter.send_request(&request).await;
            if adapter.get_circuit_status().await == CircuitState::Open {
                circuit_opened = true;
                break;
            }
        }
        assert!(circuit_opened, "Circuit breaker should have opened");
        assert!(!adapter.is_available().await);
        
        // Reset the circuit breaker
        adapter.reset_circuit_breaker().await;
        assert_eq!(adapter.get_circuit_status().await, CircuitState::Closed);
        assert!(adapter.is_available().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_state() {
        let config = Arc::new(create_test_config());
        let adapter = ExternalRpcAdapter::new(config);
        let request = create_test_request();
        
        // Open the circuit breaker
        for _ in 0..5 {
            let _ = adapter.send_request(&request).await;
        }
        assert_eq!(adapter.get_circuit_status().await, CircuitState::Open);
        
        // Wait for recovery timeout (we'll use a shorter timeout for testing)
        // In a real scenario, this would be 60 seconds
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // The circuit breaker should still be open since we haven't reached the real timeout
        assert_eq!(adapter.get_circuit_status().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_daemon_availability_tracking() {
        let config = Arc::new(create_test_config());
        let adapter = ExternalRpcAdapter::new(config);
        let request = create_test_request();
        
        // Initially available
        assert!(adapter.is_available().await);
        
        // After failures, should become unavailable
        for _ in 0..5 {
            let _ = adapter.send_request(&request).await;
        }
        
        assert!(!adapter.is_available().await);
    }
} 