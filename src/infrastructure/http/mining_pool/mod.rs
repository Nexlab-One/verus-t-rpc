//! Mining pool logic module
//! 
//! This module contains mining pool specific business logic including utilities
//! and response handling for mining pool operations.
//! 
//! This module is separate from:
//! - `handlers/mining_pool.rs` - HTTP endpoint handlers
//! - `routes/mining_pool.rs` - Route configuration
//! - `processors/base.rs` - Common request processing patterns

pub mod utils;
pub mod response_handler;

pub use utils::MiningPoolUtils;
pub use response_handler::MiningPoolResponseHandler;
