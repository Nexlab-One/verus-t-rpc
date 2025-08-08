use serde::{Deserialize, Serialize};
use serde_json::Value;

/// RPC method definition with validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMethodDefinition {
    pub name: String,
    pub description: String,
    pub read_only: bool,
    pub required_permissions: Vec<String>,
    pub parameter_rules: Vec<ParameterValidationRule>,
    pub security_level: SecurityLevel,
    pub enabled: bool,
}

/// Parameter validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidationRule {
    pub index: usize,
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub constraints: Vec<ValidationConstraint>,
    pub default_value: Option<Value>,
}

/// Parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Any,
}

/// Validation constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationConstraint {
    MinLength(usize),
    MaxLength(usize),
    MinValue(f64),
    MaxValue(f64),
    Pattern(String),
    Enum(Vec<String>),
    Custom(String),
}

/// Security levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
}


