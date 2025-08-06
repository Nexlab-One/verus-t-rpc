//! CORS configuration for reverse proxy deployment
//! 
//! This module provides CORS configuration validation for deployment
//! behind a reverse proxy (nginx, Caddy, etc.) that handles CORS.
//! 
//! DEPRECATION NOTICE: CORS should be handled by the reverse proxy
//! for better performance and more flexible configuration.

use crate::config::AppConfig;
use tracing::info;

/// CORS configuration for reverse proxy deployment
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub origins: Vec<String>,
    pub methods: Vec<String>,
    pub headers: Vec<String>,
    pub reverse_proxy_mode: bool,
}

impl CorsConfig {
    /// Create a new CORS configuration
    pub fn new(origins: Vec<String>, methods: Vec<String>, headers: Vec<String>) -> Self {
        Self {
            origins,
            methods,
            headers,
            reverse_proxy_mode: true, // Default to reverse proxy mode
        }
    }

    /// Load CORS configuration from app config
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            origins: config.security.cors_origins.clone(),
            methods: config.security.cors_methods.clone(),
            headers: config.security.cors_headers.clone(),
            reverse_proxy_mode: true, // Always use reverse proxy mode
        }
    }
}

/// CORS middleware for reverse proxy deployment
pub struct CorsMiddleware {
    config: CorsConfig,
}

impl CorsMiddleware {
    /// Create a new CORS middleware
    pub fn new(config: AppConfig) -> Self {
        let cors_config = CorsConfig::from_app_config(&config);
        
        if cors_config.reverse_proxy_mode {
            info!("CORS configuration loaded for reverse proxy deployment");
            info!("CORS headers should be configured in the reverse proxy (nginx, Caddy, etc.)");
        }

        Self { config: cors_config }
    }
    
    /// Get CORS configuration
    pub fn get_cors_config(&self) -> &CorsConfig {
        &self.config
    }
    
    /// Check if CORS allows any origin
    pub fn allows_any_origin(&self) -> bool {
        self.config.origins.contains(&"*".to_string())
    }

    /// Validate CORS configuration
    pub fn validate_config(&self) -> Result<(), String> {
        // Check if origins are valid
        if !self.allows_any_origin() {
            for origin in &self.config.origins {
                if !self.is_valid_origin(origin) {
                    return Err(format!("Invalid CORS origin: {}", origin));
                }
            }
        }

        // Check if methods are valid
        for method in &self.config.methods {
            if method.parse::<warp::http::Method>().is_err() {
                return Err(format!("Invalid CORS method: {}", method));
            }
        }

        // Check if headers are valid
        for header in &self.config.headers {
            if header.is_empty() {
                return Err(format!("Invalid CORS header: {}", header));
            }
        }

        Ok(())
    }

    /// Check if an origin is valid
    fn is_valid_origin(&self, origin: &str) -> bool {
        if origin == "*" {
            return true;
        }

        // Check for valid URL format
        if origin.starts_with("http://") || origin.starts_with("https://") {
            return true;
        }

        // Allow localhost for development
        if origin.starts_with("http://localhost:") || origin.starts_with("https://localhost:") {
            return true;
        }

        false
    }

    /// Get deployment recommendations
    pub fn get_deployment_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        recommendations.push("CORS should be handled by the reverse proxy for better performance".to_string());
        recommendations.push("Configure CORS headers in nginx/Caddy configuration".to_string());
        
        if self.allows_any_origin() {
            recommendations.push("Use 'Access-Control-Allow-Origin: *' in reverse proxy for development".to_string());
            recommendations.push("For production, specify exact origins in reverse proxy configuration".to_string());
        } else {
            recommendations.push("Configure specific origins in reverse proxy:".to_string());
            for origin in &self.config.origins {
                recommendations.push(format!("  - {}", origin));
            }
        }

        recommendations.push("Configure preflight handling in reverse proxy".to_string());
        recommendations.push("Set appropriate CORS headers for your use case".to_string());

        recommendations
    }

    /// Get CORS preflight response headers (for reference)
    pub fn get_preflight_headers(&self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        // Access-Control-Allow-Origin
        if self.allows_any_origin() {
            headers.push(("Access-Control-Allow-Origin".to_string(), "*".to_string()));
        } else {
            // For specific origins, this would be set dynamically based on the request origin
            headers.push(("Access-Control-Allow-Origin".to_string(), "null".to_string()));
        }

        // Access-Control-Allow-Methods
        let methods = self.config.methods.join(", ");
        headers.push(("Access-Control-Allow-Methods".to_string(), methods));

        // Access-Control-Allow-Headers
        let allowed_headers = self.config.headers.join(", ");
        headers.push(("Access-Control-Allow-Headers".to_string(), allowed_headers));

        // Access-Control-Max-Age
        headers.push(("Access-Control-Max-Age".to_string(), "3600".to_string()));

        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_middleware_creation() {
        let config = AppConfig::default();
        let middleware = CorsMiddleware::new(config);
        assert!(middleware.allows_any_origin());
    }

    #[test]
    fn test_cors_config_validation() {
        let config = AppConfig::default();
        let middleware = CorsMiddleware::new(config);
        assert!(middleware.validate_config().is_ok());
    }

    #[test]
    fn test_invalid_cors_method() {
        let mut config = AppConfig::default();
        config.security.cors_methods.push("INVALID METHOD".to_string());
        let middleware = CorsMiddleware::new(config);
        
        // The validation should fail because "INVALID METHOD" (with space) is not a valid HTTP method
        assert!(middleware.validate_config().is_err());
    }

    #[test]
    fn test_valid_origins() {
        let config = AppConfig::default();
        let middleware = CorsMiddleware::new(config);
        
        assert!(middleware.is_valid_origin("*"));
        assert!(middleware.is_valid_origin("http://example.com"));
        assert!(middleware.is_valid_origin("https://example.com"));
        assert!(middleware.is_valid_origin("http://localhost:3000"));
        assert!(!middleware.is_valid_origin("invalid-origin"));
    }

    #[test]
    fn test_preflight_headers() {
        let config = AppConfig::default();
        let middleware = CorsMiddleware::new(config);
        let headers = middleware.get_preflight_headers();
        
        assert!(headers.iter().any(|(k, _)| k == "Access-Control-Allow-Origin"));
        assert!(headers.iter().any(|(k, _)| k == "Access-Control-Allow-Methods"));
        assert!(headers.iter().any(|(k, _)| k == "Access-Control-Allow-Headers"));
        assert!(headers.iter().any(|(k, _)| k == "Access-Control-Max-Age"));
    }

    #[test]
    fn test_deployment_recommendations() {
        let config = AppConfig::default();
        let middleware = CorsMiddleware::new(config);
        let recommendations = middleware.get_deployment_recommendations();
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("reverse proxy")));
    }
} 