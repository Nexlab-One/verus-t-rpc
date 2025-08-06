//! Mining Pool integration adapter
//! 
//! This module handles communication with external mining pools for enhanced PoW validation.

use crate::shared::error::AppResult;
use crate::config::AppConfig;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use chrono::{Utc, DateTime};
use reqwest::Client;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::Mutex;
use ed25519_dalek::{Verifier, VerifyingKey, Signature};

/// Pool share structure for mining pool validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolShare {
    /// Challenge ID this share solves
    pub challenge_id: String,
    
    /// Miner address
    pub miner_address: String,
    
    /// Nonce used for the solution
    pub nonce: String,
    
    /// Solution hash
    pub solution: String,
    
    /// Difficulty achieved
    pub difficulty: f64,
    
    /// Timestamp when share was submitted
    pub timestamp: DateTime<Utc>,
    
    /// Optional pool signature for validation
    pub pool_signature: Option<String>,
}

/// Pool validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolValidationResponse {
    /// Whether the share is valid
    pub valid: bool,
    
    /// Pool-assigned share ID
    pub share_id: Option<String>,
    
    /// Pool signature for the validation
    pub pool_signature: Option<String>,
    
    /// Actual difficulty achieved
    pub difficulty_achieved: Option<f64>,
    
    /// Miner reputation score (0.0 to 1.0)
    pub miner_reputation: Option<f64>,
    
    /// Timestamp of validation
    pub timestamp: DateTime<Utc>,
    
    /// Error message if validation failed
    pub error: Option<String>,
}

/// Pool share submission request
#[derive(Debug, Serialize, Deserialize)]
pub struct PoolShareRequest {
    /// Challenge ID
    pub challenge_id: String,
    
    /// Miner address
    pub miner_address: String,
    
    /// Nonce used
    pub nonce: String,
    
    /// Solution hash
    pub solution: String,
    
    /// Difficulty achieved
    pub difficulty: f64,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Enhanced pool metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMetrics {
    /// Total shares submitted
    pub total_shares: u64,
    
    /// Valid shares count
    pub valid_shares: u64,
    
    /// Invalid shares count
    pub invalid_shares: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// Circuit breaker state
    pub circuit_breaker_state: CircuitBreakerState,
    
    /// Last successful validation timestamp
    pub last_success: Option<DateTime<Utc>>,
    
    /// Last error timestamp
    pub last_error: Option<DateTime<Utc>>,
    
    /// Error rate percentage
    pub error_rate_percent: f64,
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Circuit is open, requests fail fast
    HalfOpen, // Testing if service is back
}

/// Enhanced error types for better categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolError {
    /// Network connectivity issues
    NetworkError(String),
    
    /// Authentication failures
    AuthenticationError(String),
    
    /// Pool validation failures
    ValidationError(String),
    
    /// Rate limiting exceeded
    RateLimitError(String),
    
    /// Pool service unavailable
    ServiceUnavailable(String),
    
    /// Invalid response format
    InvalidResponse(String),
    
    /// Signature verification failed
    SignatureError(String),
}

/// Circuit breaker for fault tolerance
pub struct CircuitBreaker {
    state: Mutex<CircuitBreakerState>,
    failure_count: AtomicU32,
    last_failure_time: Mutex<Option<Instant>>,
    threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout_seconds: u64) -> Self {
        Self {
            state: Mutex::new(CircuitBreakerState::Closed),
            failure_count: AtomicU32::new(0),
            last_failure_time: Mutex::new(None),
            threshold,
            timeout: Duration::from_secs(timeout_seconds),
        }
    }

    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: Clone + From<crate::shared::error::AppError>,
    {
        let state = *self.state.lock().await;
        
        match state {
            CircuitBreakerState::Open => {
                let last_failure = *self.last_failure_time.lock().await;
                if let Some(last_failure_time) = last_failure {
                    if last_failure_time.elapsed() >= self.timeout {
                        // Try to transition to half-open
                        let mut state_guard = self.state.lock().await;
                        if *state_guard == CircuitBreakerState::Open {
                            *state_guard = CircuitBreakerState::HalfOpen;
                            drop(state_guard);
                            
                            // Try the call
                            match f().await {
                                Ok(result) => {
                                    // Success, close the circuit
                                    *self.state.lock().await = CircuitBreakerState::Closed;
                                    self.failure_count.store(0, Ordering::SeqCst);
                                    Ok(result)
                                }
                                Err(e) => {
                                    // Still failing, open the circuit
                                    *self.state.lock().await = CircuitBreakerState::Open;
                                    *self.last_failure_time.lock().await = Some(Instant::now());
                                    Err(e)
                                }
                            }
                        } else {
                            // Another thread already changed the state
                            Err(self.create_circuit_open_error().into())
                        }
                    } else {
                        // Circuit is still open
                        Err(self.create_circuit_open_error().into())
                    }
                } else {
                    Err(self.create_circuit_open_error().into())
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Try the call
                match f().await {
                    Ok(result) => {
                        // Success, close the circuit
                        *self.state.lock().await = CircuitBreakerState::Closed;
                        self.failure_count.store(0, Ordering::SeqCst);
                        Ok(result)
                    }
                    Err(e) => {
                        // Still failing, open the circuit
                        *self.state.lock().await = CircuitBreakerState::Open;
                        *self.last_failure_time.lock().await = Some(Instant::now());
                        Err(e)
                    }
                }
            }
            CircuitBreakerState::Closed => {
                // Normal operation
                match f().await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        // Increment failure count
                        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                        if failures >= self.threshold {
                            // Open the circuit
                            *self.state.lock().await = CircuitBreakerState::Open;
                            *self.last_failure_time.lock().await = Some(Instant::now());
                        }
                        Err(e)
                    }
                }
            }
        }
    }

    fn create_circuit_open_error(&self) -> crate::shared::error::AppError {
        crate::shared::error::AppError::Internal(
            "Mining pool service is temporarily unavailable".to_string()
        )
    }

    pub async fn get_state(&self) -> CircuitBreakerState {
        self.state.lock().await.clone()
    }
}

/// Retry mechanism with exponential backoff
pub struct RetryMechanism {
    max_retries: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl RetryMechanism {
    pub fn new(max_retries: u32, base_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_retries,
            base_delay: Duration::from_millis(base_delay_ms),
            max_delay: Duration::from_millis(max_delay_ms),
        }
    }

    pub async fn execute<F, Fut, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: Clone,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e.clone());
                    
                    if attempt < self.max_retries {
                        let delay = self.calculate_delay(attempt);
                        debug!("Pool request failed, retrying in {:?} (attempt {}/{})", delay, attempt + 1, self.max_retries + 1);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }

    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay = self.base_delay * 2_u32.pow(attempt);
        delay.min(self.max_delay)
    }
}

/// Mining Pool Client for communicating with external mining pools
pub struct MiningPoolClient {
    config: Arc<AppConfig>,
    http_client: Client,
    circuit_breaker: CircuitBreaker,
    rate_limiter: Mutex<HashMap<String, (u32, Instant)>>, // IP -> (count, window_start)
    retry_mechanism: RetryMechanism,
    metrics: Mutex<PoolMetrics>,
    pool_public_key: Option<VerifyingKey>,
}

impl MiningPoolClient {
    /// Create a new mining pool client
    pub fn new(config: Arc<AppConfig>) -> Self {
        let pool_config = config.security.mining_pool.as_ref()
            .expect("Mining pool configuration is required");
        
        let http_client = Client::builder()
            .timeout(Duration::from_secs(pool_config.timeout_seconds))
            .pool_max_idle_per_host(10) // Connection pooling
            .pool_idle_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        let circuit_breaker = CircuitBreaker::new(
            pool_config.circuit_breaker_threshold,
            pool_config.circuit_breaker_timeout,
        );
        
        let retry_mechanism = RetryMechanism::new(
            pool_config.max_retries,
            100, // 100ms base delay
            5000, // 5s max delay
        );
        
        // Parse pool public key for signature verification
        let pool_public_key = hex::decode(&pool_config.public_key)
            .ok()
            .and_then(|bytes| {
                if bytes.len() == 32 {
                    let mut key_bytes = [0u8; 32];
                    key_bytes.copy_from_slice(&bytes);
                    VerifyingKey::from_bytes(&key_bytes).ok()
                } else {
                    None
                }
            });
        
        let metrics = Mutex::new(PoolMetrics {
            total_shares: 0,
            valid_shares: 0,
            invalid_shares: 0,
            avg_response_time_ms: 0.0,
            circuit_breaker_state: CircuitBreakerState::Closed,
            last_success: None,
            last_error: None,
            error_rate_percent: 0.0,
        });
        
        Self {
            config,
            http_client,
            circuit_breaker,
            rate_limiter: Mutex::new(HashMap::new()),
            retry_mechanism,
            metrics,
            pool_public_key,
        }
    }

    /// Validate a pool share with the external mining pool
    pub async fn validate_share(&self, share: &PoolShare) -> AppResult<PoolValidationResponse> {
        let pool_config = self.config.security.mining_pool.as_ref()
            .ok_or_else(|| crate::shared::error::AppError::Internal(
                "Mining pool configuration not found".to_string()
            ))?;
        
        if !pool_config.enabled {
            return Err(crate::shared::error::AppError::Internal(
                "Mining pool integration is disabled".to_string()
            ));
        }

        // Check rate limiting
        self.check_rate_limit(&share.miner_address).await?;

        // Update metrics
        self.update_metrics_start().await;

        let start_time = Instant::now();

        // Use circuit breaker with retry mechanism
        let result = self.circuit_breaker.call(|| async {
            self.retry_mechanism.execute(|| async {
                self.submit_share_to_pool(share).await
            }).await
        }).await;

        let response_time = start_time.elapsed();
        self.update_metrics_end(result.is_ok(), response_time).await;

        result.map_err(|e| match e {
            crate::shared::error::AppError::Internal(msg) => {
                crate::shared::error::AppError::Internal(msg)
            }
            _ => crate::shared::error::AppError::Internal("Circuit breaker error".to_string())
        })
    }

    /// Submit share to the mining pool for validation
    async fn submit_share_to_pool(&self, share: &PoolShare) -> AppResult<PoolValidationResponse> {
        let pool_config = self.config.security.mining_pool.as_ref().unwrap();
        
        let request = PoolShareRequest {
            challenge_id: share.challenge_id.clone(),
            miner_address: share.miner_address.clone(),
            nonce: share.nonce.clone(),
            solution: share.solution.clone(),
            difficulty: share.difficulty,
            timestamp: share.timestamp,
        };

        let url = format!("{}/api/v1/share/validate", pool_config.pool_url);
        
        debug!("Submitting share to pool: {}", url);
        
        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", pool_config.api_key))
            .header("Content-Type", "application/json")
            .header("User-Agent", "VerusRpcServer/1.0")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to submit share to pool: {}", e);
                crate::shared::error::AppError::Internal(
                    format!("Pool communication failed: {}", e)
                )
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Pool returned error status {}: {}", status, error_text);
            return Err(crate::shared::error::AppError::Internal(
                format!("Pool validation failed: {}", error_text)
            ));
        }

        let validation_response: PoolValidationResponse = response.json().await
            .map_err(|e| {
                error!("Failed to parse pool response: {}", e);
                crate::shared::error::AppError::Internal(
                    format!("Invalid pool response: {}", e)
                )
            })?;

        // Verify pool signature if provided
        if let Some(signature) = &validation_response.pool_signature {
            if let Some(public_key) = &self.pool_public_key {
                self.verify_pool_signature(share, signature, public_key).await?;
            }
        }

        info!("Pool validation result: valid={}, share_id={:?}, reputation={:?}",
              validation_response.valid,
              validation_response.share_id,
              validation_response.miner_reputation);

        Ok(validation_response)
    }

    /// Verify pool signature for additional security
    async fn verify_pool_signature(
        &self, 
        share: &PoolShare, 
        signature: &str, 
        public_key: &VerifyingKey
    ) -> AppResult<()> {
        // Create the message to verify
        let message = format!(
            "{}:{}:{}:{}:{}:{}",
            share.challenge_id,
            share.miner_address,
            share.nonce,
            share.solution,
            share.difficulty,
            share.timestamp.timestamp()
        );
        
        let signature_bytes = hex::decode(signature)
            .map_err(|e| crate::shared::error::AppError::Validation(
                format!("Invalid signature format: {}", e)
            ))?;
        
        if signature_bytes.len() != 64 {
            return Err(crate::shared::error::AppError::Validation(
                "Invalid signature length".to_string()
            ));
        }
        
        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&signature_bytes);
        
        let signature = Signature::from_bytes(&sig_array);
        
        public_key.verify(message.as_bytes(), &signature)
            .map_err(|e| crate::shared::error::AppError::Validation(
                format!("Signature verification failed: {}", e)
            ))?;
        
        debug!("Pool signature verified successfully");
        Ok(())
    }

    /// Check rate limiting for a miner address
    async fn check_rate_limit(&self, miner_address: &str) -> AppResult<()> {
        let pool_config = self.config.security.mining_pool.as_ref().unwrap();
        let now = Instant::now();
        let window_duration = Duration::from_secs(60); // 1 minute window
        
        let mut rate_limiter = self.rate_limiter.lock().await;
        
        if let Some((count, window_start)) = rate_limiter.get_mut(miner_address) {
            if now.duration_since(*window_start) >= window_duration {
                // Reset window
                *count = 1;
                *window_start = now;
            } else {
                // Increment count
                *count += 1;
                if *count > pool_config.requests_per_minute {
                    return Err(crate::shared::error::AppError::RateLimit);
                }
            }
        } else {
            // First request for this miner
            rate_limiter.insert(miner_address.to_string(), (1, now));
        }
        
        Ok(())
    }

    /// Update metrics at start of validation
    async fn update_metrics_start(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_shares += 1;
        metrics.circuit_breaker_state = self.circuit_breaker.get_state().await;
    }

    /// Update metrics at end of validation
    async fn update_metrics_end(&self, success: bool, response_time: Duration) {
        let mut metrics = self.metrics.lock().await;
        
        if success {
            metrics.valid_shares += 1;
            metrics.last_success = Some(Utc::now());
        } else {
            metrics.invalid_shares += 1;
            metrics.last_error = Some(Utc::now());
        }
        
        // Update average response time
        let response_time_ms = response_time.as_millis() as f64;
        let total_requests = metrics.valid_shares + metrics.invalid_shares;
        metrics.avg_response_time_ms = 
            (metrics.avg_response_time_ms * (total_requests - 1) as f64 + response_time_ms) / total_requests as f64;
        
        // Update error rate
        metrics.error_rate_percent = 
            (metrics.invalid_shares as f64 / metrics.total_shares as f64) * 100.0;
    }

    /// Get circuit breaker state
    pub async fn get_circuit_breaker_state(&self) -> CircuitBreakerState {
        self.circuit_breaker.get_state().await
    }

    /// Get pool metrics
    pub async fn get_metrics(&self) -> PoolMetrics {
        let mut metrics = self.metrics.lock().await;
        metrics.circuit_breaker_state = self.circuit_breaker.get_state().await;
        metrics.clone()
    }

    /// Check if pool is available
    pub async fn health_check(&self) -> AppResult<bool> {
        let pool_config = self.config.security.mining_pool.as_ref()
            .ok_or_else(|| crate::shared::error::AppError::Internal(
                "Mining pool configuration not found".to_string()
            ))?;
        
        if !pool_config.enabled {
            return Ok(false);
        }

        let url = format!("{}/api/v1/health", pool_config.pool_url);
        
        match self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", pool_config.api_key))
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!("Pool health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get pool configuration summary
    pub fn get_config_summary(&self) -> String {
        if let Some(pool_config) = &self.config.security.mining_pool {
            format!(
                "Pool: {}, Timeout: {}s, Retries: {}, Circuit Breaker: {}/{}s",
                pool_config.pool_url,
                pool_config.timeout_seconds,
                pool_config.max_retries,
                pool_config.circuit_breaker_threshold,
                pool_config.circuit_breaker_timeout
            )
        } else {
            "Mining pool not configured".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker() {
        let circuit_breaker = CircuitBreaker::new(3, 1);
        
        // Test normal operation
        let result = circuit_breaker.call(|| async { 
            Ok::<i32, crate::shared::error::AppError>(42) 
        }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(circuit_breaker.get_state().await, CircuitBreakerState::Closed);
        
        // Test failure threshold
        for _ in 0..3 {
            let result = circuit_breaker.call(|| async { 
                Err::<i32, crate::shared::error::AppError>(
                    crate::shared::error::AppError::Internal("error".to_string())
                ) 
            }).await;
            assert!(result.is_err());
        }
        
        // Circuit should be open now
        assert_eq!(circuit_breaker.get_state().await, CircuitBreakerState::Open);
        
        // Requests should fail fast
        let result = circuit_breaker.call(|| async { 
            Ok::<i32, crate::shared::error::AppError>(42) 
        }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pool_share_serialization() {
        let share = PoolShare {
            challenge_id: "test-challenge".to_string(),
            miner_address: "test-miner".to_string(),
            nonce: "12345".to_string(),
            solution: "abcdef".to_string(),
            difficulty: 1.5,
            timestamp: Utc::now(),
            pool_signature: Some("signature".to_string()),
        };
        
        let json = serde_json::to_string(&share).unwrap();
        let deserialized: PoolShare = serde_json::from_str(&json).unwrap();
        
        assert_eq!(share.challenge_id, deserialized.challenge_id);
        assert_eq!(share.miner_address, deserialized.miner_address);
        assert_eq!(share.nonce, deserialized.nonce);
        assert_eq!(share.solution, deserialized.solution);
        assert_eq!(share.difficulty, deserialized.difficulty);
    }

    #[tokio::test]
    async fn test_retry_mechanism() {
        let retry = RetryMechanism::new(3, 10, 100);
        let attempts = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempts_clone = attempts.clone();
        
        let result = retry.execute(move || {
            let attempts = attempts_clone.clone();
            async move {
                let current = attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if current < 2 {
                    Err::<i32, crate::shared::error::AppError>(
                        crate::shared::error::AppError::Internal("temporary error".to_string())
                    )
                } else {
                    Ok(42)
                }
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_pool_metrics() {
        // Create a config with mining pool configuration
        let mut config = AppConfig::default();
        config.security.mining_pool = Some(crate::config::app_config::MiningPoolConfig {
            pool_url: "https://test-pool.com".to_string(),
            api_key: "test-key".to_string(),
            public_key: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: 60,
            requests_per_minute: 100,
            enabled: true,
        });
        
        let config = Arc::new(config);
        let client = MiningPoolClient::new(config);
        
        let metrics = client.get_metrics().await;
        assert_eq!(metrics.total_shares, 0);
        assert_eq!(metrics.valid_shares, 0);
        assert_eq!(metrics.invalid_shares, 0);
        assert_eq!(metrics.circuit_breaker_state, CircuitBreakerState::Closed);
    }
}
