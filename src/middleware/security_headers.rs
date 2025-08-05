use crate::config::AppConfig;
use warp::Filter;

/// Create security headers layer based on configuration
pub fn create_security_layer(config: &AppConfig) -> impl Filter<Extract = (), Error = warp::reject::Rejection> + Clone {
    if !config.security.enable_security_headers {
        return warp::any().map(|| ());
    }

    warp::any()
        .map(|| ())
} 