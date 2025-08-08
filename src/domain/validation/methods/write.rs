use crate::domain::validation::types::{RpcMethodDefinition, SecurityLevel, ParameterValidationRule, ParameterType, ValidationConstraint};
use crate::domain::validation::registry::MethodRegistry;

pub fn register_write(registry: &mut MethodRegistry) {
    registry.register_method(RpcMethodDefinition {
        name: "sendrawtransaction".to_string(),
        description: "Send a raw transaction".to_string(),
        read_only: false,
        required_permissions: vec!["send_transaction".to_string()],
        parameter_rules: vec![
            ParameterValidationRule {
                index: 0,
                name: "hexstring".to_string(),
                param_type: ParameterType::String,
                required: true,
                constraints: vec![
                    ValidationConstraint::MinLength(1),
                    ValidationConstraint::MaxLength(100000),
                ],
                default_value: None,
            },
        ],
        security_level: SecurityLevel::High,
        enabled: true,
    });
}


