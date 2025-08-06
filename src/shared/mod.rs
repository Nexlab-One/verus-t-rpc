//! Shared utilities and common functionality
//! 
//! This module contains shared utilities, error handling, logging,
//! metrics, and validation that are used across the application.

pub mod error;
pub mod logging;
pub mod metrics;
pub mod validation;

pub use error::{AppError, AppResult};
pub use logging::LoggingUtils;
pub use metrics::MetricsUtils;
pub use validation::ValidationUtils; 