//! Parameter validation helpers for RpcService

use crate::{
    domain::rpc::{Constraint, ParameterRule},
    shared::error::{AppError, AppResult},
};
use serde_json::Value;
use tracing::warn;

pub fn validate_parameter_rule(rule: &ParameterRule, parameters: &Value) -> AppResult<()> {
    let param_value = match parameters {
        Value::Array(arr) => {
            if rule.index < arr.len() {
                Some(&arr[rule.index])
            } else if rule.required {
                return Err(AppError::InvalidParameters {
                    method: rule.name.clone(),
                    reason: format!("Required parameter at index {} not found", rule.index),
                });
            } else {
                None
            }
        }
        Value::Object(obj) => obj.get(&rule.name),
        _ => {
            if rule.required {
                return Err(AppError::InvalidParameters {
                    method: rule.name.clone(),
                    reason: "Parameters must be array or object".to_string(),
                });
            } else {
                None
            }
        }
    };

    if let Some(value) = param_value {
        validate_parameter_value(rule, value)?;
    }

    Ok(())
}

pub fn validate_parameter_value(rule: &ParameterRule, value: &Value) -> AppResult<()> {
    for constraint in &rule.constraints {
        match constraint {
            Constraint::MinLength(min_len) => {
                if let Value::String(s) = value {
                    if s.len() < *min_len {
                        return Err(AppError::InvalidParameters {
                            method: rule.name.clone(),
                            reason: format!("Parameter {} too short (min {} characters)", rule.name, min_len),
                        });
                    }
                }
            }
            Constraint::MaxLength(max_len) => {
                if let Value::String(s) = value {
                    if s.len() > *max_len {
                        return Err(AppError::InvalidParameters {
                            method: rule.name.clone(),
                            reason: format!("Parameter {} too long (max {} characters)", rule.name, max_len),
                        });
                    }
                }
            }
            Constraint::Pattern(pattern) => {
                if let Value::String(s) = value {
                    use regex::Regex;
                    match Regex::new(pattern) {
                        Ok(regex) => {
                            if !regex.is_match(s) {
                                return Err(AppError::InvalidParameters {
                                    method: rule.name.clone(),
                                    reason: format!("Parameter {} does not match pattern {}", rule.name, pattern),
                                });
                            }
                        }
                        Err(e) => {
                            return Err(AppError::Validation(format!(
                                "Invalid regex pattern '{}': {}",
                                pattern, e
                            )));
                        }
                    }
                }
            }
            _ => {
                warn!("Constraint validation not yet implemented: {:?}", constraint);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde_json::json;

    fn rule_string_min_len(name: &str, min: usize, required: bool) -> ParameterRule {
        ParameterRule {
            index: 0,
            name: name.to_string(),
            param_type: crate::domain::rpc::ParameterType::String,
            required,
            constraints: vec![Constraint::MinLength(min)],
        }
    }

    #[test]
    fn validate_parameter_rule_array_ok() {
        let rule = rule_string_min_len("arg0", 3, true);
        let params = json!(["abcd"]);
        assert!(validate_parameter_rule(&rule, &params).is_ok());
    }

    #[test]
    fn validate_parameter_rule_array_missing_required_err() {
        let rule = rule_string_min_len("arg0", 3, true);
        let params = json!([]);
        let res = validate_parameter_rule(&rule, &params);
        assert!(res.is_err());
    }

    #[test]
    fn validate_parameter_rule_object_ok() {
        let rule = rule_string_min_len("name", 2, true);
        let params = json!({"name": "ok"});
        assert!(validate_parameter_rule(&rule, &params).is_ok());
    }

    #[test]
    fn validate_parameter_value_pattern() {
        let rule = ParameterRule {
            index: 0,
            name: "hex".to_string(),
            param_type: crate::domain::rpc::ParameterType::String,
            required: true,
            constraints: vec![Constraint::Pattern("^[0-9a-f]+$".to_string())],
        };
        assert!(validate_parameter_value(&rule, &json!("deadbeef")).is_ok());
        assert!(validate_parameter_value(&rule, &json!("DEADBEEF")).is_err());
    }
}


