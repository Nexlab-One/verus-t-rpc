use crate::domain::validation::types::{RpcMethodDefinition, SecurityLevel, ParameterValidationRule, ParameterType, ValidationConstraint};
use crate::domain::validation::registry::MethodRegistry;

pub fn register_currency(registry: &mut MethodRegistry) {
    registry.register_method(RpcMethodDefinition {
        name: "getcurrency".to_string(),
        description: "Get currency information".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![
            ParameterValidationRule {
                index: 0,
                name: "currency".to_string(),
                param_type: ParameterType::String,
                required: true,
                constraints: vec![
                    ValidationConstraint::MinLength(1),
                    ValidationConstraint::MaxLength(100),
                ],
                default_value: None,
            },
        ],
        security_level: SecurityLevel::Medium,
        enabled: true,
    });
}


