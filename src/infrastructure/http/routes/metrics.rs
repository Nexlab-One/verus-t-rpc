//! Metrics routes module
//! 
//! This module contains metrics and Prometheus route configurations.

use crate::{
    config::AppConfig,
    infrastructure::http::{
        utils::{with_metrics_use_case, with_config, with_prometheus_adapter},
        handlers::{handle_metrics_request, handle_prometheus_request},
    },
    application::use_cases::GetMetricsUseCase,
};
use std::sync::Arc;
use warp::Filter;

/// Metrics routes configuration
pub struct MetricsRoutes;

impl MetricsRoutes {
    /// Create the metrics endpoint route
    pub fn create_metrics_route(
        config: AppConfig,
        metrics_use_case: Arc<GetMetricsUseCase>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("metrics")
            .and(warp::get())
            .and(with_metrics_use_case(metrics_use_case))
            .and(with_config(config))
            .and_then(handle_metrics_request)
    }

    /// Create the Prometheus metrics endpoint route
    pub fn create_prometheus_route(
        config: AppConfig,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("metrics")
            .and(warp::path("prometheus"))
            .and(warp::get())
            .and(with_prometheus_adapter())
            .and(with_config(config))
            .and_then(handle_prometheus_request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::services::MetricsService;
    use serde_json::Value;

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    fn create_test_metrics_use_case() -> Arc<GetMetricsUseCase> {
        let metrics_service = Arc::new(MetricsService::new());
        Arc::new(GetMetricsUseCase::new(metrics_service))
    }

    #[test]
    fn test_metrics_routes_create_metrics_route() {
        let config = create_test_config();
        let metrics_use_case = create_test_metrics_use_case();

        // This should not panic and should return a valid filter
        let route = MetricsRoutes::create_metrics_route(
            config,
            metrics_use_case,
        );
        let _ = route.clone();
    }

    #[test]
    fn test_metrics_routes_create_prometheus_route() {
        let config = create_test_config();

        // This should not panic and should return a valid filter
        let route = MetricsRoutes::create_prometheus_route(config);
        let _ = route.clone();
    }

    #[test]
    fn test_metrics_routes_with_different_configs() {
        let mut config = create_test_config();
        config.server.port = 9091; // Different port
        
        let metrics_use_case = create_test_metrics_use_case();

        // Should work with different config values
        let metrics_route = MetricsRoutes::create_metrics_route(
            config.clone(),
            metrics_use_case,
        );

        let prometheus_route = MetricsRoutes::create_prometheus_route(config);
        let _ = metrics_route.clone();
        let _ = prometheus_route.clone();
    }

    #[test]
    fn test_metrics_routes_route_structure() {
        let config = create_test_config();
        let metrics_use_case = create_test_metrics_use_case();

        // Test that the routes are created with the expected structure
        let metrics_route = MetricsRoutes::create_metrics_route(
            config.clone(),
            metrics_use_case,
        );

        let prometheus_route = MetricsRoutes::create_prometheus_route(config);
        let _ = metrics_route.clone();
        let _ = prometheus_route.clone();
    }

    #[tokio::test]
    async fn test_metrics_route_e2e_status_headers_body() {
        let config = create_test_config();
        let metrics_use_case = create_test_metrics_use_case();
        let route = MetricsRoutes::create_metrics_route(config, metrics_use_case);

        let res = warp::test::request()
            .method("GET")
            .path("/metrics")
            .reply(&route)
            .await;

        assert_eq!(res.status(), warp::http::StatusCode::OK);
        assert!(res.headers().contains_key("content-security-policy"));
        let body: Value = serde_json::from_slice(res.body()).unwrap();
        assert!(body.get("total_requests").is_some());
    }

    #[tokio::test]
    async fn test_prometheus_route_e2e_status_headers_content_type() {
        let config = create_test_config();
        let route = MetricsRoutes::create_prometheus_route(config);

        let res = warp::test::request()
            .method("GET")
            .path("/metrics/prometheus")
            .reply(&route)
            .await;

        assert_eq!(res.status(), warp::http::StatusCode::OK);
        assert!(res.headers().contains_key("content-security-policy"));
        assert_eq!(
            res.headers().get("content-type").unwrap().to_str().unwrap(),
            "text/plain; version=0.0.4; charset=utf-8"
        );
        let text = std::str::from_utf8(res.body()).unwrap();
        assert!(text.contains("# HELP"));
    }
}
