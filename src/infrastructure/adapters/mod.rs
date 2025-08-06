//! This module contains adapters for external services and infrastructure concerns.

pub mod authentication;
pub mod cache;
pub mod comprehensive_validator;
pub mod external_rpc;
pub mod monitoring;
pub mod token_issuer;
pub mod mining_pool;

pub use authentication::AuthenticationAdapter;
pub use cache::{CacheAdapter, CacheConfig, CacheEntry, CacheStats};
pub use comprehensive_validator::ComprehensiveValidator;
pub use external_rpc::ExternalRpcAdapter;
pub use monitoring::{MonitoringAdapter, MetricsEvent, MetricsSummary};
pub use token_issuer::{
    TokenIssuerAdapter, TokenIssuanceRequest, TokenIssuanceResponse,
    TokenValidationRequest, TokenValidationResponse, JwtClaims,
    TokenIssuanceMode, PowProof, PowChallenge, PowAlgorithm, PowManager
};
pub use mining_pool::{
    MiningPoolClient, PoolShare, PoolValidationResponse, PoolShareRequest,
    CircuitBreaker, CircuitBreakerState
}; 