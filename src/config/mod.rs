//! Configuration management module
//! 
//! This module handles all configuration concerns including loading,
//! validation, and providing access to application settings.

pub mod app_config;
pub mod validation;

pub use app_config::AppConfig;
pub use validation::ConfigValidator; 