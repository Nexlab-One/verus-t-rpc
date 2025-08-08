//! HTTP route handlers module
//! 
//! This module contains separate route handlers for different endpoint types,
//! organized by functionality to improve maintainability and testability.

pub mod rpc;
pub mod health;
pub mod metrics;
pub mod mining_pool;
pub mod payments;

pub use rpc::handle_rpc_request;
pub use health::handle_health_request;
pub use metrics::{handle_metrics_request, handle_prometheus_request};
pub use mining_pool::{handle_mining_pool_request, handle_pool_metrics_request};
pub use payments::{handle_payment_quote, handle_payment_submit, handle_payment_status};
