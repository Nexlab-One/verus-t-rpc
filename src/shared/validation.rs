//! Validation utilities module
//! 
//! This module provides common validation functionality used across the application.

use serde_json::Value;
// use validator::Validate;
use crate::shared::error::AppError;

/// Validation utilities for the application
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate JSON-RPC method name
    pub fn validate_method_name(method: &str) -> crate::Result<()> {
        if method.is_empty() {
            return Err(AppError::Validation(
                "Method name cannot be empty".to_string()
            ));
        }

        if method.len() > 100 {
            return Err(crate::shared::error::AppError::Validation(
                "Method name too long (max 100 characters)".to_string()
            ));
        }

        // Check for valid characters (alphanumeric, underscore, dot)
        if !method.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
            return Err(crate::shared::error::AppError::Validation(
                "Method name contains invalid characters".to_string()
            ));
        }

        Ok(())
    }

    /// Validate JSON-RPC parameters
    pub fn validate_parameters(params: &Value) -> crate::Result<()> {
        match params {
            Value::Array(arr) => {
                if arr.len() > 100 {
                    return Err(crate::shared::error::AppError::Validation(
                        "Too many parameters (max 100)".to_string()
                    ));
                }
                
                // Validate each parameter
                for (i, param) in arr.iter().enumerate() {
                    Self::validate_parameter(param, i)?;
                }
            }
            Value::Object(obj) => {
                if obj.len() > 50 {
                    return Err(crate::shared::error::AppError::Validation(
                        "Too many parameter keys (max 50)".to_string()
                    ));
                }
                
                // Validate each parameter
                for (key, value) in obj {
                    if key.len() > 100 {
                        return Err(crate::shared::error::AppError::Validation(
                            format!("Parameter key too long: {}", key)
                        ));
                    }
                    Self::validate_parameter(value, 0)?;
                }
            }
            Value::Null => {
                // Null is valid
            }
            _ => {
                return Err(crate::shared::error::AppError::Validation(
                    "Parameters must be an array, object, or null".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Validate individual parameter
    fn validate_parameter(param: &Value, index: usize) -> crate::Result<()> {
        match param {
            Value::String(s) => {
                if s.len() > 10000 {
                    return Err(crate::shared::error::AppError::Validation(
                        format!("Parameter {} string too long (max 10000 characters)", index)
                    ));
                }
            }
            Value::Number(_n) => {
                // Check if number is reasonable
                if let Some(f) = _n.as_f64() {
                    if !f.is_finite() {
                        return Err(crate::shared::error::AppError::Validation(
                            format!("Parameter {} is not a finite number", index)
                        ));
                    }
                }
            }
            Value::Array(arr) => {
                if arr.len() > 1000 {
                    return Err(crate::shared::error::AppError::Validation(
                        format!("Parameter {} array too large (max 1000 elements)", index)
                    ));
                }
                
                for (i, item) in arr.iter().enumerate() {
                    Self::validate_parameter(item, i)?;
                }
            }
            Value::Object(obj) => {
                if obj.len() > 100 {
                    return Err(crate::shared::error::AppError::Validation(
                        format!("Parameter {} object too large (max 100 keys)", index)
                    ));
                }
                
                for (key, value) in obj {
                    if key.len() > 100 {
                        return Err(crate::shared::error::AppError::Validation(
                            format!("Parameter {} object key too long: {}", index, key)
                        ));
                    }
                    Self::validate_parameter(value, 0)?;
                }
            }
            Value::Bool(_) | Value::Null => {
                // These are always valid
            }
        }

        Ok(())
    }

    /// Validate request ID
    pub fn validate_request_id(id: &Value) -> crate::Result<()> {
        match id {
            Value::String(s) => {
                if s.len() > 100 {
                    return Err(crate::shared::error::AppError::Validation(
                        "Request ID string too long (max 100 characters)".to_string()
                    ));
                }
            }
            Value::Number(_n) => {
                // Numbers are valid
            }
            Value::Null => {
                // Null is valid
            }
            _ => {
                return Err(crate::shared::error::AppError::Validation(
                    "Request ID must be a string, number, or null".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Validate client IP address
    pub fn validate_client_ip(ip: &str) -> crate::Result<()> {
        if ip.is_empty() {
            return Err(crate::shared::error::AppError::Validation(
                "Client IP cannot be empty".to_string()
            ));
        }

        if ip.len() > 45 {
            return Err(crate::shared::error::AppError::Validation(
                "Client IP too long".to_string()
            ));
        }

        // Basic IP format validation
        if !ip.contains('.') && !ip.contains(':') {
            return Err(crate::shared::error::AppError::Validation(
                "Invalid IP address format".to_string()
            ));
        }

        Ok(())
    }

    /// Validate user agent
    pub fn validate_user_agent(user_agent: &str) -> crate::Result<()> {
        if user_agent.len() > 500 {
            return Err(crate::shared::error::AppError::Validation(
                "User agent too long (max 500 characters)".to_string()
            ));
        }

        Ok(())
    }
} 