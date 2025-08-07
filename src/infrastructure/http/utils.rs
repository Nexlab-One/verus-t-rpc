//! HTTP utilities - Common helper functions
//! 
//! This module contains utility functions used across the HTTP infrastructure
//! for IP validation, route injection, and other common operations.

use crate::config::AppConfig;
use crate::shared::error::AppResult;
use crate::application::use_cases::{ProcessRpcRequestUseCase, GetMetricsUseCase, HealthCheckUseCase};
use crate::middleware::{cache::CacheMiddleware, rate_limit::RateLimitMiddleware};
use std::sync::Arc;
use warp::Filter;

/// Extract and validate client IP from various sources
pub fn extract_and_validate_client_ip(raw_ip: &str, config: &AppConfig) -> String {
    // If the IP is empty or invalid, return a default
    if raw_ip.is_empty() || raw_ip == "unknown" {
        return "127.0.0.1".to_string();
    }
    
    // Parse the IP to validate it
    if let Ok(ip) = raw_ip.parse::<std::net::IpAddr>() {
        // Check if it's a private/local IP and if we should trust it
        if config.security.trusted_proxy_headers.contains(&"X-Forwarded-For".to_string()) {
            // If we trust proxy headers, return the IP as-is
            return ip.to_string();
        } else {
            // If we don't trust proxy headers, only accept local IPs
            if ip.is_loopback() {
                return ip.to_string();
            } else {
                return "127.0.0.1".to_string();
            }
        }
    }
    
    // If parsing failed, return default
    "127.0.0.1".to_string()
}

/// Parse pool share from domain request parameters
pub fn parse_pool_share_from_request(domain_request: &crate::domain::rpc::RpcRequest) -> AppResult<crate::infrastructure::adapters::PoolShare> {
    use crate::shared::error::AppError;
    use serde_json::Value;
    
    // Extract parameters from the domain request
    let params = domain_request.parameters.as_ref()
        .ok_or_else(|| AppError::Validation("Missing request parameters".to_string()))?;
    
    // Parse the parameters as a JSON object
    let params_obj = params.as_object()
        .ok_or_else(|| AppError::Validation("Parameters must be a JSON object".to_string()))?;
    
    // Extract required fields with validation
    let challenge_id = params_obj.get("challenge_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing or invalid challenge_id".to_string()))?
        .to_string();
    
    let miner_address = params_obj.get("miner_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing or invalid miner_address".to_string()))?
        .to_string();
    
    let nonce = params_obj.get("nonce")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing or invalid nonce".to_string()))?
        .to_string();
    
    let solution = params_obj.get("solution")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing or invalid solution".to_string()))?
        .to_string();
    
    let difficulty = params_obj.get("difficulty")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| AppError::Validation("Missing or invalid difficulty".to_string()))?;
    
    // Parse timestamp (accept both ISO string and Unix timestamp)
    let timestamp = if let Some(timestamp_value) = params_obj.get("timestamp") {
        match timestamp_value {
            Value::String(timestamp_str) => {
                // Try to parse as ISO 8601 string
                chrono::DateTime::parse_from_rfc3339(timestamp_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .or_else(|_| {
                        // Try to parse as Unix timestamp
                        timestamp_str.parse::<i64>()
                            .ok()
                            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                            .ok_or_else(|| AppError::Validation("Invalid timestamp format".to_string()))
                    })?
            }
            Value::Number(timestamp_num) => {
                // Parse as Unix timestamp
                timestamp_num.as_i64()
                    .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                    .ok_or_else(|| AppError::Validation("Invalid timestamp value".to_string()))?
            }
            _ => return Err(AppError::Validation("Invalid timestamp format".to_string())),
        }
    } else {
        // Use current timestamp if not provided
        chrono::Utc::now()
    };
    
    // Parse optional pool signature
    let pool_signature = params_obj.get("pool_signature")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // Validate field lengths and values
    if challenge_id.is_empty() {
        return Err(AppError::Validation("challenge_id cannot be empty".to_string()));
    }
    if miner_address.is_empty() {
        return Err(AppError::Validation("miner_address cannot be empty".to_string()));
    }
    if nonce.is_empty() {
        return Err(AppError::Validation("nonce cannot be empty".to_string()));
    }
    if solution.is_empty() {
        return Err(AppError::Validation("solution cannot be empty".to_string()));
    }
    if difficulty <= 0.0 {
        return Err(AppError::Validation("difficulty must be positive".to_string()));
    }
    
    Ok(crate::infrastructure::adapters::PoolShare {
        challenge_id,
        miner_address,
        nonce,
        solution,
        difficulty,
        timestamp,
        pool_signature,
    })
}

/// Helper function to inject RPC use case into route
pub fn with_rpc_use_case(
    rpc_use_case: Arc<ProcessRpcRequestUseCase>,
) -> impl Filter<Extract = (Arc<ProcessRpcRequestUseCase>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || rpc_use_case.clone())
}

/// Helper function to inject health use case into route
pub fn with_health_use_case(
    health_use_case: Arc<HealthCheckUseCase>,
) -> impl Filter<Extract = (Arc<HealthCheckUseCase>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || health_use_case.clone())
}

/// Helper function to inject metrics use case into route
pub fn with_metrics_use_case(
    metrics_use_case: Arc<GetMetricsUseCase>,
) -> impl Filter<Extract = (Arc<GetMetricsUseCase>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || metrics_use_case.clone())
}

/// Helper function to inject mining pool client into route
pub fn with_mining_pool_client(
) -> impl Filter<Extract = (Arc<crate::infrastructure::adapters::MiningPoolClient>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || {
        // Create a default config for the mining pool client
        let mut config = crate::config::AppConfig::default();
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
        Arc::new(crate::infrastructure::adapters::MiningPoolClient::new(Arc::new(config)))
    })
}

/// Helper function to inject Prometheus adapter into route
pub fn with_prometheus_adapter(
) -> impl Filter<Extract = (Arc<crate::infrastructure::adapters::MonitoringAdapter>,), Error = std::convert::Infallible> + Clone {
    let monitoring_adapter = Arc::new(crate::infrastructure::adapters::MonitoringAdapter::new());
    warp::any().map(move || monitoring_adapter.clone())
}

/// Helper function to inject configuration into route
pub fn with_config(
    config: AppConfig,
) -> impl Filter<Extract = (AppConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

/// Helper function to inject cache middleware into route
pub fn with_cache_middleware(
    cache_middleware: Arc<CacheMiddleware>,
) -> impl Filter<Extract = (Arc<CacheMiddleware>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || cache_middleware.clone())
}

/// Helper function to inject rate limiting middleware into route
pub fn with_rate_limit_middleware(
    rate_limit_middleware: Arc<RateLimitMiddleware>,
) -> impl Filter<Extract = (Arc<RateLimitMiddleware>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || rate_limit_middleware.clone())
}
