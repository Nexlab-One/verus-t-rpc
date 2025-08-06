//! Configuration validation module
//! 
//! This module provides additional validation logic for configuration
//! beyond the basic validator crate validation.

use crate::config::AppConfig;
use crate::shared::error::AppError;

/// Configuration validator for additional validation logic
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate the complete configuration
    pub fn validate_config(config: &AppConfig) -> crate::Result<()> {
        // Validate Verus RPC URL format
        Self::validate_verus_url(&config.verus.rpc_url)?;
        
        // Validate security settings
        Self::validate_security_config(&config.security)?;
        
        // Validate rate limiting settings
        Self::validate_rate_limit_config(&config.rate_limit)?;
        
        Ok(())
    }
    
    /// Validate Verus RPC URL
    fn validate_verus_url(url: &str) -> crate::Result<()> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(AppError::Validation(
                "Verus RPC URL must start with http:// or https://".to_string()
            ));
        }
        
        if url.contains("localhost") || url.contains("127.0.0.1") {
            // Allow localhost for development
            Ok(())
        } else {
            // For production, ensure HTTPS
            if !url.starts_with("https://") {
                return Err(AppError::Validation(
                    "Production Verus RPC URL must use HTTPS".to_string()
                ));
            }
            Ok(())
        }
    }
    
    /// Validate security configuration
    fn validate_security_config(security: &crate::config::app_config::SecurityConfig) -> crate::Result<()> {
        // Check for overly permissive CORS settings
        if security.cors_origins.contains(&"*".to_string()) && security.enable_security_headers {
            tracing::warn!("CORS is configured to allow any origin - this may be a security risk in production");
        }
        
        // Validate CORS methods
        for method in &security.cors_methods {
            if !["GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH"].contains(&method.as_str()) {
                return Err(AppError::Validation(
                    format!("Invalid CORS method: {}", method)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate rate limiting configuration
    fn validate_rate_limit_config(rate_limit: &crate::config::app_config::RateLimitConfig) -> crate::Result<()> {
        if rate_limit.enabled {
            if rate_limit.requests_per_minute == 0 {
                return Err(AppError::Validation(
                    "Rate limiting enabled but requests_per_minute is 0".to_string()
                ));
            }
            
            if rate_limit.burst_size > rate_limit.requests_per_minute {
                return Err(AppError::Validation(
                    "Burst size cannot be greater than requests per minute".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::app_config::{SecurityConfig, RateLimitConfig};
    use std::collections::HashMap;

    #[test]
    fn test_validate_verus_url_valid_http() {
        let result = ConfigValidator::validate_verus_url("http://localhost:8080");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_verus_url_valid_https() {
        let result = ConfigValidator::validate_verus_url("https://api.verus.io");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_verus_url_invalid_protocol() {
        let result = ConfigValidator::validate_verus_url("ftp://localhost:8080");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must start with http:// or https://"));
    }

    #[test]
    fn test_validate_verus_url_production_requires_https() {
        let result = ConfigValidator::validate_verus_url("http://api.verus.io");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must use HTTPS"));
    }

    #[test]
    fn test_validate_security_config_valid() {
        let security = SecurityConfig {
            cors_origins: vec!["https://example.com".to_string()],
            cors_methods: vec!["GET".to_string(), "POST".to_string()],
            cors_headers: vec!["Content-Type".to_string()],
            enable_request_logging: true,
            enable_security_headers: true,
            trusted_proxy_headers: vec!["X-Forwarded-For".to_string()],
            enable_custom_headers: true,
            custom_security_header: Some("X-Custom-Header".to_string()),
            method_rate_limits: HashMap::new(),
            jwt: crate::config::app_config::JwtConfig {
                secret_key: "your-super-secret-jwt-key-that-is-at-least-32-characters-long".to_string(),
                expiration_seconds: 3600,
                issuer: "verus-rpc-server".to_string(),
                audience: "verus-clients".to_string(),
            },
            pow: None,
            mining_pool: None,
            development_mode: false,
        };
        
        let result = ConfigValidator::validate_security_config(&security);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_security_config_invalid_method() {
        let security = SecurityConfig {
            cors_origins: vec![],
            cors_methods: vec!["INVALID".to_string()],
            cors_headers: vec![],
            enable_request_logging: false,
            enable_security_headers: false,
            trusted_proxy_headers: vec![],
            enable_custom_headers: false,
            custom_security_header: None,
            method_rate_limits: std::collections::HashMap::new(),
            jwt: crate::config::app_config::JwtConfig {
                secret_key: "test_secret_key_32_chars_long".to_string(),
                expiration_seconds: 3600,
                issuer: "test".to_string(),
                audience: "test".to_string(),
            },
            pow: None,
            mining_pool: None,
            development_mode: false,
        };
        
        let result = ConfigValidator::validate_security_config(&security);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid CORS method"));
    }

    #[test]
    fn test_validate_rate_limit_config_valid() {
        let rate_limit = RateLimitConfig {
            requests_per_minute: 100,
            burst_size: 50,
            enabled: true,
        };
        
        let result = ConfigValidator::validate_rate_limit_config(&rate_limit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rate_limit_config_zero_requests() {
        let rate_limit = RateLimitConfig {
            requests_per_minute: 0,
            burst_size: 50,
            enabled: true,
        };
        
        let result = ConfigValidator::validate_rate_limit_config(&rate_limit);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requests_per_minute is 0"));
    }

    #[test]
    fn test_validate_rate_limit_config_burst_too_large() {
        let rate_limit = RateLimitConfig {
            requests_per_minute: 100,
            burst_size: 150,
            enabled: true,
        };
        
        let result = ConfigValidator::validate_rate_limit_config(&rate_limit);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Burst size cannot be greater"));
    }

    #[test]
    fn test_validate_rate_limit_config_disabled() {
        let rate_limit = RateLimitConfig {
            requests_per_minute: 100,
            burst_size: 50,
            enabled: false,
        };
        let result = ConfigValidator::validate_rate_limit_config(&rate_limit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_complete() {
        let config = AppConfig::default();
        let result = ConfigValidator::validate_config(&config);
        assert!(result.is_ok());
    }
} 