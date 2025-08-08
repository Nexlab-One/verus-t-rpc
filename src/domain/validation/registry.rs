use std::collections::HashMap;
use serde_json::{Value, value::RawValue};
use crate::shared::error::AppResult;
use super::types::{
    RpcMethodDefinition,
    ParameterValidationRule,
    ParameterType,
    ValidationConstraint,
};
use super::methods::{
    core::register_core,
    blocks::register_blocks,
    transactions::register_transactions,
    write::register_write,
    identity::register_identity,
    currency::register_currency,
    utility::register_utility,
    additional::register_additional_methods,
};

/// Method registry for RPC validation
pub struct MethodRegistry {
    pub(crate) methods: HashMap<String, RpcMethodDefinition>,
}

impl MethodRegistry {
    /// Create a new method registry
    pub fn new() -> Self {
        let mut registry = Self {
            methods: HashMap::new(),
        };

        // Register all supported methods
        registry.register_default_methods();

        registry
    }

    /// Register a method definition
    pub fn register_method(&mut self, method: RpcMethodDefinition) {
        self.methods.insert(method.name.clone(), method);
    }

    /// Get a method definition
    pub fn get_method(&self, name: &str) -> Option<&RpcMethodDefinition> {
        self.methods.get(name)
    }

    /// Check if a method is allowed
    pub fn is_method_allowed(&self, name: &str) -> bool {
        self.methods.get(name)
            .map(|method| method.enabled)
            .unwrap_or(false)
    }

    /// Validate method parameters
    pub fn validate_method_parameters(&self, method_name: &str, params: &[Box<RawValue>]) -> AppResult<()> {
        let method = self.methods.get(method_name)
            .ok_or_else(|| crate::shared::error::AppError::MethodNotAllowed {
                method: method_name.to_string(),
            })?;

        // Check parameter count
        if params.len() > method.parameter_rules.len() {
            return Err(crate::shared::error::AppError::InvalidParameters {
                method: method_name.to_string(),
                reason: format!("Too many parameters: expected {}, got {}", method.parameter_rules.len(), params.len()),
            });
        }

        // Validate each parameter
        for (i, rule) in method.parameter_rules.iter().enumerate() {
            if i < params.len() {
                self.validate_parameter(&params[i], rule)?;
            } else if rule.required {
                return Err(crate::shared::error::AppError::InvalidParameters {
                    method: method_name.to_string(),
                    reason: format!("Missing required parameter: {}", rule.name),
                });
            }
        }

        Ok(())
    }

    /// Validate a single parameter
    fn validate_parameter(&self, param: &RawValue, rule: &ParameterValidationRule) -> AppResult<()> {
        let value: Value = serde_json::from_str(&param.to_string())
            .map_err(|e| crate::shared::error::AppError::InvalidParameters {
                method: "unknown".to_string(),
                reason: format!("Invalid JSON in parameter {}: {}", rule.name, e),
            })?;

        // Check type
        if !self.matches_type(&value, &rule.param_type) {
            return Err(crate::shared::error::AppError::InvalidParameters {
                method: "unknown".to_string(),
                reason: format!("Parameter {} has wrong type", rule.name),
            });
        }

        // Apply constraints
        for constraint in &rule.constraints {
            self.apply_constraint(&value, constraint, &rule.name)?;
        }

        Ok(())
    }

    /// Check if value matches parameter type
    fn matches_type(&self, value: &Value, param_type: &ParameterType) -> bool {
        match param_type {
            ParameterType::String => matches!(value, Value::String(_)),
            ParameterType::Number => matches!(value, Value::Number(_)),
            ParameterType::Boolean => matches!(value, Value::Bool(_)),
            ParameterType::Object => matches!(value, Value::Object(_)),
            ParameterType::Array => matches!(value, Value::Array(_)),
            ParameterType::Any => true,
        }
    }

    /// Apply validation constraint
    fn apply_constraint(&self, value: &Value, constraint: &ValidationConstraint, param_name: &str) -> AppResult<()> {
        match constraint {
            ValidationConstraint::MinLength(min_len) => {
                if let Value::String(s) = value {
                    if s.len() < *min_len {
                        return Err(crate::shared::error::AppError::InvalidParameters {
                            method: "unknown".to_string(),
                            reason: format!("Parameter {} too short: minimum length is {}", param_name, min_len),
                        });
                    }
                }
            },
            ValidationConstraint::MaxLength(max_len) => {
                if let Value::String(s) = value {
                    if s.len() > *max_len {
                        return Err(crate::shared::error::AppError::InvalidParameters {
                            method: "unknown".to_string(),
                            reason: format!("Parameter {} too long: maximum length is {}", param_name, max_len),
                        });
                    }
                }
            },
            ValidationConstraint::MinValue(min_val) => {
                if let Value::Number(n) = value {
                    if let Some(f) = n.as_f64() {
                        if f < *min_val {
                            return Err(crate::shared::error::AppError::InvalidParameters {
                                method: "unknown".to_string(),
                                reason: format!("Parameter {} too small: minimum value is {}", param_name, min_val),
                            });
                        }
                    }
                }
            },
            ValidationConstraint::MaxValue(max_val) => {
                if let Value::Number(n) = value {
                    if let Some(f) = n.as_f64() {
                        if f > *max_val {
                            return Err(crate::shared::error::AppError::InvalidParameters {
                                method: "unknown".to_string(),
                                reason: format!("Parameter {} too large: maximum value is {}", param_name, max_val),
                            });
                        }
                    }
                }
            },
            ValidationConstraint::Pattern(pattern) => {
                if let Value::String(s) = value {
                    use regex::Regex;
                    match Regex::new(pattern) {
                        Ok(regex) => {
                            if !regex.is_match(s) {
                                return Err(crate::shared::error::AppError::InvalidParameters {
                                    method: "unknown".to_string(),
                                    reason: format!("Parameter {} doesn't match pattern: {}", param_name, pattern),
                                });
                            }
                        }
                        Err(e) => {
                            return Err(crate::shared::error::AppError::Validation(
                                format!("Invalid regex pattern '{}': {}", pattern, e)
                            ));
                        }
                    }
                }
            },
            ValidationConstraint::Enum(allowed_values) => {
                if let Value::String(s) = value {
                    if !allowed_values.contains(s) {
                        return Err(crate::shared::error::AppError::InvalidParameters {
                            method: "unknown".to_string(),
                            reason: format!("Parameter {} must be one of: {:?}", param_name, allowed_values),
                        });
                    }
                }
            },
            ValidationConstraint::Custom(validation_name) => {
                match validation_name.as_str() {
                    "hex_string" => {
                        if let Value::String(s) = value {
                            if !s.chars().all(|c| c.is_ascii_hexdigit()) {
                                return Err(crate::shared::error::AppError::InvalidParameters {
                                    method: "unknown".to_string(),
                                    reason: format!("Parameter {} must be a valid hex string", param_name),
                                });
                            }
                        }
                    },
                    "base58_string" => {
                        if let Value::String(s) = value {
                            if !s.chars().all(|c| c.is_alphanumeric() && !"0OIl".contains(c)) {
                                return Err(crate::shared::error::AppError::InvalidParameters {
                                    method: "unknown".to_string(),
                                    reason: format!("Parameter {} must be a valid Base58 string", param_name),
                                });
                            }
                        }
                    },
                    "block_hash" => {
                        if let Value::String(s) = value {
                            if s.len() != 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
                                return Err(crate::shared::error::AppError::InvalidParameters {
                                    method: "unknown".to_string(),
                                    reason: format!("Parameter {} must be a valid 64-character hex block hash", param_name),
                                });
                            }
                        }
                    },
                    _ => {
                        return Err(crate::shared::error::AppError::Validation(
                            format!("Unknown custom validation: {}", validation_name)
                        ));
                    }
                }
            },
        }

        Ok(())
    }

    /// Register default methods
    fn register_default_methods(&mut self) {
        // Register all Verus RPC methods with comprehensive validation
        self.register_verus_methods_modular();
    }

    /// Modular registration that delegates to submodules
    fn register_verus_methods_modular(&mut self) {
        register_core(self);
        register_blocks(self);
        register_transactions(self);
        register_write(self);
        register_identity(self);
        register_currency(self);
        register_utility(self);
        register_additional_methods(self);
    }

}

impl Default for MethodRegistry {
    fn default() -> Self { Self::new() }
}


