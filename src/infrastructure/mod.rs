//! Infrastructure layer - External concerns and adapters
//! 
//! This module contains infrastructure concerns including external services,
//! adapters, converters, and HTTP handling.

pub mod adapters;
pub mod converters;
pub mod http;

// Re-export main adapters
pub use adapters::{ComprehensiveValidator, ExternalRpcAdapter, AuthenticationAdapter, MonitoringAdapter};
pub use converters::*; 