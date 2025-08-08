use serde_json::{Value, value::RawValue};
use crate::shared::error::AppResult;
use super::registry::MethodRegistry;
use super::types::RpcMethodDefinition;

/// Domain validator that uses the method registry
pub struct DomainValidator {
    registry: MethodRegistry,
}

impl DomainValidator {
    /// Create a new domain validator
    pub fn new() -> Self {
        Self { registry: MethodRegistry::new() }
    }

    /// Validate a method call
    pub fn validate_method_call(&self, method: &str, params: &Option<Value>) -> AppResult<()> {
        // Check if method is allowed
        if !self.registry.is_method_allowed(method) {
            return Err(crate::shared::error::AppError::MethodNotAllowed { method: method.to_string() });
        }

        // Convert params to raw format for validation
        let raw_params: Vec<Box<RawValue>> = if let Some(params) = params {
            if let Some(array) = params.as_array() {
                array
                    .iter()
                    .map(|v| RawValue::from_string(v.to_string())
                        .map_err(|e| crate::shared::error::AppError::Internal(format!("Failed to create raw value: {}", e))))
                    .collect::<AppResult<Vec<Box<RawValue>>>>()?
            } else {
                return Err(crate::shared::error::AppError::InvalidParameters {
                    method: method.to_string(),
                    reason: "Parameters must be an array".to_string(),
                });
            }
        } else {
            vec![]
        };

        // Validate parameters
        self.registry.validate_method_parameters(method, &raw_params)?;

        Ok(())
    }

    /// Get method definition
    pub fn get_method_definition(&self, method: &str) -> Option<&RpcMethodDefinition> {
        self.registry.get_method(method)
    }

    /// Check if method is read-only
    pub fn is_method_read_only(&self, method: &str) -> bool {
        self.registry
            .get_method(method)
            .map(|m| m.read_only)
            .unwrap_or(true)
    }

    /// Get required permissions for a method
    pub fn get_required_permissions(&self, method: &str) -> Vec<String> {
        self.registry
            .get_method(method)
            .map(|m| m.required_permissions.clone())
            .unwrap_or_default()
    }
}

impl Default for DomainValidator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validate_getinfo_ok() {
        let validator = DomainValidator::new();
        let params: Option<Value> = None;
        assert!(validator.validate_method_call("getinfo", &params).is_ok());
    }

    #[test]
    fn validate_invalid_method_err() {
        let validator = DomainValidator::new();
        let params: Option<Value> = None;
        assert!(validator.validate_method_call("not_a_method", &params).is_err());
    }

    #[test]
    fn validate_missing_required_param_err() {
        let validator = DomainValidator::new();
        let params = Some(json!([]));
        assert!(validator.validate_method_call("getblock", &params).is_err());
    }

    #[test]
    fn validate_too_many_params_err() {
        let validator = DomainValidator::new();
        let hash = "0".repeat(64);
        let params = Some(json!([hash, true, "extra"]));
        assert!(validator.validate_method_call("getblock", &params).is_err());
    }

    #[test]
    fn validate_wrong_type_err() {
        let validator = DomainValidator::new();
        let params = Some(json!([123]));
        assert!(validator.validate_method_call("getblock", &params).is_err());
    }

    #[test]
    fn validate_constraint_length_err() {
        let validator = DomainValidator::new();
        let params = Some(json!(["abcd"]));
        assert!(validator.validate_method_call("getblock", &params).is_err());
    }

    #[test]
    fn validate_getrawtransaction_verbose_out_of_range_err() {
        let validator = DomainValidator::new();
        let txid = "a".repeat(64);
        let params = Some(json!([txid, 2]));
        assert!(validator.validate_method_call("getrawtransaction", &params).is_err());
    }

    #[test]
    fn validate_getrawtransaction_verbose_ok() {
        let validator = DomainValidator::new();
        let txid = "b".repeat(64);
        let params = Some(json!([txid, 1]));
        assert!(validator.validate_method_call("getrawtransaction", &params).is_ok());
    }

    #[test]
    fn read_only_flags() {
        let validator = DomainValidator::new();
        assert!(validator.is_method_read_only("getinfo"));
        assert!(!validator.is_method_read_only("sendrawtransaction"));
    }

    #[test]
    fn required_permissions_fetch() {
        let validator = DomainValidator::new();
        let perms = validator.get_required_permissions("sendrawtransaction");
        assert_eq!(perms, vec!["send_transaction".to_string()]);
    }

    #[test]
    fn method_definition_is_available() {
        let validator = DomainValidator::new();
        let def = validator.get_method_definition("getblock");
        assert!(def.is_some());
        assert_eq!(def.unwrap().name, "getblock".to_string());
    }

    #[test]
    fn enum_constraint_for_optional_param() {
        let validator = DomainValidator::new();
        // Invalid enum value should error
        let params_invalid = Some(json!(["foo"]));
        assert!(validator.validate_method_call("z_getnewaddress", &params_invalid).is_err());

        // Valid enum value should pass
        let params_valid = Some(json!(["sapling"]));
        assert!(validator.validate_method_call("z_getnewaddress", &params_valid).is_ok());
    }
}


