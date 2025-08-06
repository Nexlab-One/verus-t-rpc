use crate::config::AppConfig;
use crate::shared::error::AppError;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::warn;
use warp::{Rejection, Reply};

/// Rate limiting configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub enabled: bool,
}

/// Rate limiting state for a client
#[derive(Clone)]
pub struct ClientRateLimit {
    pub requests: u32,
    pub window_start: u64,
}

/// Rate limiting state
pub struct RateLimitState {
    clients: Arc<RwLock<HashMap<String, ClientRateLimit>>>,
    config: RateLimitConfig,
}

impl RateLimitState {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Check if request is allowed
    pub async fn check_rate_limit(&self, key: &str) -> Result<(), AppError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let window_start = now - (now % 60); // 1-minute windows
        
        let mut clients = self.clients.write().await;
        
        if let Some(client) = clients.get_mut(key) {
            if client.window_start != window_start {
                // New window, reset counter
                client.requests = 1;
                client.window_start = window_start;
            } else if client.requests >= self.config.requests_per_minute {
                // Rate limit exceeded
                warn!("Rate limit exceeded for key: {}", key);
                return Err(AppError::RateLimit);
            } else {
                // Increment counter
                client.requests += 1;
            }
        } else {
            // New client
            clients.insert(key.to_string(), ClientRateLimit {
                requests: 1,
                window_start,
            });
        }
        
        Ok(())
    }
}

/// Rate limiting middleware for HTTP responses
pub struct RateLimitMiddleware {
    config: AppConfig,
}

impl RateLimitMiddleware {
    /// Create a new rate limiting middleware
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
    
    /// Get rate limiting configuration
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
    
    /// Check if rate limiting is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.rate_limit.enabled
    }
    
    /// Create a rate limiter for a specific client
    pub fn create_client_limiter(&self, _client_ip: &str) -> RateLimitState {
        RateLimitState::new(RateLimitConfig {
            requests_per_minute: self.config.rate_limit.requests_per_minute,
            burst_size: self.config.rate_limit.burst_size,
            enabled: self.config.rate_limit.enabled,
        })
    }
}

/// Rate limiting middleware for specific endpoints
pub async fn rate_limit_middleware<T: Reply>(
    response: T,
    rate_limit_state: &RateLimitState,
    client_ip: &str,
) -> Result<T, AppError> {
    rate_limit_state.check_rate_limit(client_ip).await?;
    Ok(response)
}

/// Create a rate limiter for specific methods
pub fn create_method_rate_limiter(
    method: &str,
    config: &AppConfig,
) -> RateLimitState {
    let method_config = config.security.method_rate_limits.get(method)
        .unwrap_or(&config.rate_limit);
    
    RateLimitState::new(RateLimitConfig {
        requests_per_minute: method_config.requests_per_minute,
        burst_size: method_config.burst_size,
        enabled: method_config.enabled,
    })
}

/// Rate limiting error handler
pub fn handle_rate_limit_error(err: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(app_error) = err.find::<AppError>() {
        if matches!(app_error, AppError::RateLimit) {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": {
                        "code": -429,
                        "message": "Rate limit exceeded"
                    }
                })),
                warp::http::StatusCode::TOO_MANY_REQUESTS,
            ));
        }
    }
    
    // Return a default error response
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "error": {
                "code": -500,
                "message": "Internal server error"
            }
        })),
        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
    ))
} 