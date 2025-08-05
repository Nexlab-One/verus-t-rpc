use crate::config::AppConfig;
use governor::{Quota, RateLimiter, DefaultKeyedRateLimiter, DefaultDirectRateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;
use warp::Filter;

/// Create rate limiting layer based on configuration
pub fn create_rate_limit_layer(config: &AppConfig) -> impl Filter<Extract = (), Error = warp::reject::Rejection> + Clone {
    if !config.rate_limit.enabled {
        return warp::any().map(|| ());
    }

    let limiter = Arc::new(RateLimiter::direct(Quota::per_minute(
        NonZeroU32::new(config.rate_limit.requests_per_minute).unwrap(),
    )));

    warp::any()
        .and(with_rate_limiter(limiter))
        .and_then(rate_limit_check)
}

async fn rate_limit_check(
    limiter: Arc<DefaultDirectRateLimiter>,
) -> Result<(), warp::reject::Rejection> {
    // TODO: Extract client IP from request
    let client_ip = "127.0.0.1"; // Placeholder
    
    if limiter.check().is_err() {
        return Err(warp::reject::custom(crate::error::AppError::RateLimit));
    }
    
    Ok(())
}

fn with_rate_limiter(
    limiter: Arc<DefaultDirectRateLimiter>,
) -> impl Filter<Extract = (Arc<DefaultDirectRateLimiter>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || limiter.clone())
} 