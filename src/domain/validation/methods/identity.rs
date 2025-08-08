use serde_json::Value;
use crate::domain::validation::types::{RpcMethodDefinition, SecurityLevel, ParameterValidationRule, ParameterType, ValidationConstraint};
use crate::domain::validation::registry::MethodRegistry;

pub fn register_identity(registry: &mut MethodRegistry) {
    registry.register_method(RpcMethodDefinition {
        name: "getidentity".to_string(),
        description: "Get identity information".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![
            ParameterValidationRule {
                index: 0,
                name: "identity".to_string(),
                param_type: ParameterType::String,
                required: true,
                constraints: vec![
                    ValidationConstraint::MinLength(1),
                    ValidationConstraint::MaxLength(100),
                ],
                default_value: None,
            },
            ParameterValidationRule {
                index: 1,
                name: "height".to_string(),
                param_type: ParameterType::Number,
                required: false,
                constraints: vec![
                    ValidationConstraint::MinValue(0.0),
                ],
                default_value: Some(Value::Number(serde_json::Number::from(-1))),
            },
            ParameterValidationRule {
                index: 2,
                name: "txproof".to_string(),
                param_type: ParameterType::Boolean,
                required: false,
                constraints: vec![],
                default_value: Some(Value::Bool(false)),
            },
            ParameterValidationRule {
                index: 3,
                name: "txproofheight".to_string(),
                param_type: ParameterType::Number,
                required: false,
                constraints: vec![
                    ValidationConstraint::MinValue(0.0),
                ],
                default_value: Some(Value::Number(serde_json::Number::from(-1))),
            },
        ],
        security_level: SecurityLevel::Medium,
        enabled: true,
    });
}


