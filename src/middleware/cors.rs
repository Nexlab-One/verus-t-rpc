use crate::config::AppConfig;

/// Create CORS layer based on configuration
pub fn create_cors_layer(_config: &AppConfig) -> impl warp::Filter + Clone {
    warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec!["Content-Type", "Authorization"])
        .max_age(3600)
} 