use serde_json::Value;
use crate::domain::validation::types::{RpcMethodDefinition, SecurityLevel, ParameterValidationRule, ParameterType, ValidationConstraint};
use crate::domain::validation::registry::MethodRegistry;

pub fn register_transactions(registry: &mut MethodRegistry) {
    registry.register_method(RpcMethodDefinition {
        name: "getrawtransaction".to_string(),
        description: "Get raw transaction".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![
            ParameterValidationRule {
                index: 0,
                name: "txid".to_string(),
                param_type: ParameterType::String,
                required: true,
                constraints: vec![
                    ValidationConstraint::MinLength(64),
                    ValidationConstraint::MaxLength(64),
                ],
                default_value: None,
            },
            ParameterValidationRule {
                index: 1,
                name: "verbose".to_string(),
                param_type: ParameterType::Number,
                required: false,
                constraints: vec![
                    ValidationConstraint::MinValue(0.0),
                    ValidationConstraint::MaxValue(1.0),
                ],
                default_value: Some(Value::Number(serde_json::Number::from(0))),
            },
        ],
        security_level: SecurityLevel::Low,
        enabled: true,
    });
}


