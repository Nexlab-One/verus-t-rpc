//! Infrastructure adapters module
//! 
//! This module contains adapters for external services and infrastructure concerns.

pub mod comprehensive_validator;
pub mod external_rpc;
pub mod authentication;
pub mod monitoring;
pub mod cache;
pub mod token_issuer;

// Re-export all adapters
pub use comprehensive_validator::ComprehensiveValidator;
pub use external_rpc::ExternalRpcAdapter;
pub use authentication::AuthenticationAdapter;
pub use monitoring::{MonitoringAdapter, MetricsEvent, MetricsSummary};
pub use cache::{CacheAdapter, CacheConfig, CacheEntry, CacheStats};
pub use token_issuer::{
    TokenIssuerAdapter, TokenIssuanceRequest, TokenIssuanceResponse,
    TokenValidationRequest, TokenValidationResponse, JwtClaims
}; 