//! HTTP routes module
//! 
//! This module contains route configuration and building logic to organize
//! the application's HTTP endpoints in a maintainable way.

pub mod builder;
pub mod config;
pub mod middleware_builder;
pub mod fluent_builder;
pub mod rpc;
pub mod health;
pub mod metrics;
pub mod mining_pool;

pub use builder::RouteBuilder;
pub use config::{RouteConfig, RpcRouteConfig, HealthRouteConfig, MetricsRouteConfig, MiningPoolRouteConfig};
pub use middleware_builder::{MiddlewareConfig, MiddlewareUtils};
pub use fluent_builder::{FluentRouteBuilder, FluentRouteUtils};
pub use rpc::RpcRoutes;
pub use health::HealthRoutes;
pub use metrics::MetricsRoutes;
pub use mining_pool::MiningPoolRoutes;
