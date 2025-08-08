use serde_json::Value;
use crate::domain::validation::types::{RpcMethodDefinition, SecurityLevel, ParameterValidationRule, ParameterType, ValidationConstraint};
use crate::domain::validation::registry::MethodRegistry;

pub fn register_blocks(registry: &mut MethodRegistry) {
    registry.register_method(RpcMethodDefinition {
        name: "getblock".to_string(),
        description: "Get block information".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![
            ParameterValidationRule {
                index: 0,
                name: "hash".to_string(),
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
                param_type: ParameterType::Boolean,
                required: false,
                constraints: vec![],
                default_value: Some(Value::Bool(true)),
            },
        ],
        security_level: SecurityLevel::Low,
        enabled: true,
    });

    registry.register_method(RpcMethodDefinition {
        name: "getblockhash".to_string(),
        description: "Get block hash by height".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![
            ParameterValidationRule {
                index: 0,
                name: "height".to_string(),
                param_type: ParameterType::Number,
                required: true,
                constraints: vec![
                    ValidationConstraint::MinValue(0.0),
                ],
                default_value: None,
            },
        ],
        security_level: SecurityLevel::Low,
        enabled: true,
    });

    registry.register_method(RpcMethodDefinition {
        name: "getblockheader".to_string(),
        description: "Get block header".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![
            ParameterValidationRule {
                index: 0,
                name: "hash".to_string(),
                param_type: ParameterType::String,
                required: true,
                constraints: vec![
                    ValidationConstraint::MinLength(64),
                    ValidationConstraint::MaxLength(64),
                ],
                default_value: None,
            },
        ],
        security_level: SecurityLevel::Low,
        enabled: true,
    });
}


