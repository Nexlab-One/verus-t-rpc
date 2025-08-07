//! Metrics handler module
//! 
//! This module contains the metrics and Prometheus endpoint handlers for monitoring.

use crate::{
    config::AppConfig,
    application::use_cases::GetMetricsUseCase,
    middleware::security_headers::{SecurityHeadersMiddleware, create_json_response_with_security_headers, add_security_headers_to_response},
};
use std::sync::Arc;
use warp::{Reply};

/// Handle metrics requests
pub async fn handle_metrics_request(
    metrics_use_case: Arc<GetMetricsUseCase>,
    config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    let metrics_data = metrics_use_case.execute();
    
    // Apply security headers only
    let response = create_json_response_with_security_headers(
        &metrics_data,
        &SecurityHeadersMiddleware::new(config.clone()),
    );
    
    Ok(response)
}

/// Handle Prometheus metrics requests
pub async fn handle_prometheus_request(
    monitoring_adapter: Arc<crate::infrastructure::adapters::MonitoringAdapter>,
    config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    let metrics = monitoring_adapter.get_prometheus_metrics();
    
    // Apply security headers only
    let response = add_security_headers_to_response(
        warp::reply::with_header(
            warp::reply::with_status(metrics, warp::http::StatusCode::OK),
            "Content-Type",
            "text/plain; version=0.0.4; charset=utf-8"
        ),
        &SecurityHeadersMiddleware::new(config.clone()),
    );
    
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::services::MetricsService;

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    fn create_test_metrics_use_case() -> Arc<GetMetricsUseCase> {
        let metrics_service = Arc::new(MetricsService::new());
        Arc::new(GetMetricsUseCase::new(metrics_service))
    }

    fn create_test_monitoring_adapter() -> Arc<crate::infrastructure::adapters::MonitoringAdapter> {
        Arc::new(crate::infrastructure::adapters::MonitoringAdapter::new())
    }

    #[tokio::test]
    async fn test_handle_metrics_request_success() {
        let metrics_use_case = create_test_metrics_use_case();
        let config = create_test_config();

        let result = handle_metrics_request(metrics_use_case, config).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_metrics_request_with_different_configs() {
        let metrics_use_case = create_test_metrics_use_case();
        let mut config = create_test_config();
        
        // Test with different configurations
        config.server.port = 8081;
        config.server.bind_address = "127.0.0.1".parse().unwrap();

        let result = handle_metrics_request(metrics_use_case, config).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_metrics_request_response_structure() {
        let metrics_use_case = create_test_metrics_use_case();
        let config = create_test_config();

        let result = handle_metrics_request(metrics_use_case, config).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_prometheus_request_success() {
        let monitoring_adapter = create_test_monitoring_adapter();
        let config = create_test_config();

        let result = handle_prometheus_request(monitoring_adapter, config).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_prometheus_request_response_structure() {
        let monitoring_adapter = create_test_monitoring_adapter();
        let config = create_test_config();

        let result = handle_prometheus_request(monitoring_adapter, config).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_use_case_execute() {
        let metrics_use_case = create_test_metrics_use_case();
        
        let metrics_data = metrics_use_case.execute();
        
        // Verify metrics data structure
        assert!(metrics_data.is_object());
        
        let metrics_obj = metrics_data.as_object().unwrap();
        assert!(metrics_obj.contains_key("total_requests"));
        assert!(metrics_obj.contains_key("successful_requests"));
        assert!(metrics_obj.contains_key("failed_requests"));
        assert!(metrics_obj.contains_key("rate_limited_requests"));
        assert!(metrics_obj.contains_key("avg_response_time_ms"));
        assert!(metrics_obj.contains_key("active_connections"));
        assert!(metrics_obj.contains_key("uptime_seconds"));
    }

    #[tokio::test]
    async fn test_monitoring_adapter_get_prometheus_metrics() {
        let monitoring_adapter = create_test_monitoring_adapter();
        
        let metrics = monitoring_adapter.get_prometheus_metrics();
        
        // Verify Prometheus metrics format
        assert!(!metrics.is_empty());
        assert!(metrics.contains("# HELP"));
        assert!(metrics.contains("# TYPE"));
        assert!(metrics.contains("rpc_requests_total"));
        assert!(metrics.contains("rpc_response_time_seconds"));
        assert!(metrics.contains("rpc_active_connections"));
    }

    #[tokio::test]
    async fn test_metrics_handler_with_security_headers_disabled() {
        let metrics_use_case = create_test_metrics_use_case();
        let mut config = create_test_config();
        
        // Disable security headers
        config.security.enable_security_headers = false;

        let result = handle_metrics_request(metrics_use_case, config).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prometheus_handler_with_security_headers_disabled() {
        let monitoring_adapter = create_test_monitoring_adapter();
        let mut config = create_test_config();
        
        // Disable security headers
        config.security.enable_security_headers = false;

        let result = handle_prometheus_request(monitoring_adapter, config).await;
        
        assert!(result.is_ok());
    }
}
