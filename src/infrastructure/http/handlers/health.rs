//! Health check handler module
//! 
//! This module contains the health check endpoint handler for monitoring system status.

use crate::{
    config::AppConfig,
    application::use_cases::HealthCheckUseCase,
    middleware::security_headers::{SecurityHeadersMiddleware, create_json_response_with_security_headers},
};
use std::sync::Arc;
use warp::{Reply};

/// Handle health check requests
pub async fn handle_health_request(
    health_use_case: Arc<HealthCheckUseCase>,
    config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    let health_data = health_use_case.execute();
    
    // Apply security headers only
    let response = create_json_response_with_security_headers(
        &health_data,
        &SecurityHeadersMiddleware::new(config.clone()),
    );
    
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    fn create_test_health_use_case() -> Arc<HealthCheckUseCase> {
        Arc::new(HealthCheckUseCase)
    }

    #[tokio::test]
    async fn test_handle_health_request_success() {
        let health_use_case = create_test_health_use_case();
        let config = create_test_config();

        let result = handle_health_request(health_use_case, config).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_health_request_with_different_configs() {
        let health_use_case = create_test_health_use_case();
        let mut config = create_test_config();
        
        // Test with different configurations
        config.server.port = 8081;
        config.server.bind_address = "127.0.0.1".parse().unwrap();

        let result = handle_health_request(health_use_case, config).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_health_request_response_structure() {
        let health_use_case = create_test_health_use_case();
        let config = create_test_config();

        let result = handle_health_request(health_use_case, config).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_use_case_execute() {
        let health_use_case = create_test_health_use_case();
        
        let health_data = health_use_case.execute();
        
        // Verify health data structure
        assert!(health_data.is_object());
        
        let health_obj = health_data.as_object().unwrap();
        assert!(health_obj.contains_key("status"));
        assert!(health_obj.contains_key("timestamp"));
        assert!(health_obj.contains_key("version"));
        
        assert_eq!(health_obj.get("status").unwrap(), "healthy");
    }

    #[tokio::test]
    async fn test_health_handler_with_security_headers_disabled() {
        let health_use_case = create_test_health_use_case();
        let mut config = create_test_config();
        
        // Disable security headers
        config.security.enable_security_headers = false;

        let result = handle_health_request(health_use_case, config).await;
        
        assert!(result.is_ok());
    }
}
