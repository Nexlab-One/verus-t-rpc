//! HTTP routes module
//! 
//! This module contains all HTTP route configurations and handlers.

pub mod builder;
pub mod fluent_builder;
pub mod middleware_builder;
pub mod rpc;
pub mod metrics;
pub mod mining_pool;
pub mod payments;

// Re-export commonly used types
pub use builder::RouteBuilder;
pub use fluent_builder::FluentRouteBuilder;
pub use middleware_builder::MiddlewareConfig;
pub use rpc::RpcRoutes;
pub use metrics::MetricsRoutes;
pub use mining_pool::MiningPoolRoutes;
pub use payments::PaymentsRoutes;
