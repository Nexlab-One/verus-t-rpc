//! Health routes module
//! 
//! This module contains health check route configurations.

use crate::{
    config::AppConfig,
    infrastructure::http::{
        utils::{with_health_use_case, with_config},
        handlers::handle_health_request,
    },
    application::use_cases::HealthCheckUseCase,
};
use std::sync::Arc;
use warp::Filter;

/// Health routes configuration
pub struct HealthRoutes;

impl HealthRoutes {
    /// Create the health check endpoint route
    pub fn create_health_route(
        config: AppConfig,
        health_use_case: Arc<HealthCheckUseCase>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("health")
            .and(warp::get())
            .and(with_health_use_case(health_use_case))
            .and(with_config(config))
            .and_then(handle_health_request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::use_cases::HealthCheckUseCase;
    use serde_json::Value;

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    fn create_test_health_use_case() -> Arc<HealthCheckUseCase> {
        Arc::new(HealthCheckUseCase)
    }

    #[test]
    fn test_health_routes_creation() {
        let config = create_test_config();
        let health_use_case = create_test_health_use_case();

        let route = HealthRoutes::create_health_route(
            config,
            health_use_case,
        );
        let _ = route.clone();
    }

    #[test]
    fn test_health_routes_with_different_configs() {
        let mut config = create_test_config();
        let health_use_case = create_test_health_use_case();

        // Test with different configurations
        config.server.port = 8081;
        config.server.bind_address = "127.0.0.1".parse().unwrap();

        let route = HealthRoutes::create_health_route(
            config,
            health_use_case,
        );
        let _ = route.clone();
    }

    #[tokio::test]
    async fn test_health_route_e2e_status_headers_body() {
        let config = create_test_config();
        let health_use_case = create_test_health_use_case();

        let route = HealthRoutes::create_health_route(config, health_use_case);

        let res = warp::test::request()
            .method("GET")
            .path("/health")
            .reply(&route)
            .await;

        assert_eq!(res.status(), warp::http::StatusCode::OK);
        assert!(res.headers().contains_key("content-security-policy"));
        let body: Value = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(body["status"], "healthy");
        assert!(body.get("timestamp").is_some());
    }
}
