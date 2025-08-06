//! Logging utilities module
//! 
//! This module provides centralized logging functionality and utilities.

use tracing::{error, info, warn};
use std::time::{SystemTime, UNIX_EPOCH};

/// Logging utilities for the application
pub struct LoggingUtils;

impl LoggingUtils {
    /// Initialize logging with the specified configuration
    pub fn initialize(level: &str, _format: &str, _structured: bool) -> crate::Result<()> {
        use tracing_subscriber::{fmt, EnvFilter};

        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level));

        let subscriber_builder = fmt::Subscriber::builder()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(false);

        let subscriber = subscriber_builder.finish();

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| crate::shared::error::AppError::Internal(format!("Failed to initialize logging: {}", e)))?;
        
        Ok(())
    }

    /// Log a request with structured data
    pub fn log_request(
        request_id: &str,
        method: &str,
        client_ip: &str,
        user_agent: Option<&str>,
        params: Option<&serde_json::Value>,
    ) {
        info!(
            request_id = %request_id,
            method = %method,
            client_ip = %client_ip,
            user_agent = user_agent,
            params = ?params,
            "Processing RPC request"
        );
    }

    /// Log a successful response
    pub fn log_success(request_id: &str, method: &str, duration_ms: u64) {
        info!(
            request_id = %request_id,
            method = %method,
            duration_ms = %duration_ms,
            "Request completed successfully"
        );
    }

    /// Log an error response
    pub fn log_error(request_id: &str, method: &str, error: &crate::shared::error::AppError, duration_ms: u64) {
        error!(
            request_id = %request_id,
            method = %method,
            error = %error,
            duration_ms = %duration_ms,
            "Request failed"
        );
    }

    /// Log security events
    pub fn log_security_event(event_type: &str, details: &str, client_ip: &str) {
        warn!(
            event_type = %event_type,
            details = %details,
            client_ip = %client_ip,
            "Security event detected"
        );
    }

    /// Log rate limiting events
    pub fn log_rate_limit(client_ip: &str, current: u32, limit: u32) {
        warn!(
            client_ip = %client_ip,
            current_requests = %current,
            limit = %limit,
            "Rate limit exceeded"
        );
    }

    /// Generate a unique request ID
    pub fn generate_request_id() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        
        format!("req_{:x}", now)
    }
} 