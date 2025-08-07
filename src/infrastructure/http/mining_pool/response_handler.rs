//! Mining pool response handler module
//! 
//! This module contains mining pool specific response handling logic.

use crate::{
    infrastructure::http::models::{JsonRpcResponse, JsonRpcError},
    infrastructure::adapters::PoolValidationResponse,
    shared::error::AppError,
};
use tracing::info;

/// Mining pool response handler
pub struct MiningPoolResponseHandler;

impl MiningPoolResponseHandler {
    /// Create a JSON-RPC response from pool validation response
    pub fn create_pool_validation_response(
        domain_response: &PoolValidationResponse,
        request_id: &str,
    ) -> JsonRpcResponse {
        // Convert domain response to JSON value
        let result = serde_json::to_value(domain_response)
            .unwrap_or_else(|_| serde_json::json!({
                "error": "Failed to serialize pool validation response"
            }));
        
        JsonRpcResponse::success(result, Some(serde_json::Value::String(request_id.to_string())))
    }

    /// Handle successful pool validation
    pub fn handle_successful_validation(
        domain_response: &PoolValidationResponse,
        request_id: &str,
        context_request_id: &str,
    ) -> JsonRpcResponse {
        info!(
            request_id = %context_request_id,
            share_id = ?domain_response.share_id,
            reputation = ?domain_response.miner_reputation,
            "Mining pool share validation successful"
        );
        
        Self::create_pool_validation_response(domain_response, request_id)
    }

    /// Handle failed pool validation
    pub fn handle_failed_validation(
        error: &AppError,
        request_id: &str,
        context_request_id: &str,
    ) -> JsonRpcResponse {
        info!(
            request_id = %context_request_id,
            error = %error,
            "Mining pool share validation failed"
        );
        
        JsonRpcResponse::error(
            JsonRpcError::internal_error(&error.to_string()),
            Some(serde_json::Value::String(request_id.to_string())),
        )
    }

    /// Validate pool validation response
    pub fn validate_pool_response(response: &PoolValidationResponse) -> Result<(), AppError> {
        if !response.valid {
            return Err(AppError::Validation(
                response.error.clone().unwrap_or_else(|| 
                    "Pool share validation failed".to_string()
                )
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_pool_validation_response(valid: bool, error: Option<String>) -> PoolValidationResponse {
        PoolValidationResponse {
            valid,
            share_id: Some("test_share_id".to_string()),
            pool_signature: Some("test_signature".to_string()),
            difficulty_achieved: Some(1.5),
            miner_reputation: Some(0.85),
            timestamp: chrono::Utc::now(),
            error,
        }
    }

    #[test]
    fn test_create_pool_validation_response_success() {
        let response = create_test_pool_validation_response(true, None);
        let request_id = "test_request_123";
        
        let result = MiningPoolResponseHandler::create_pool_validation_response(&response, request_id);
        
        assert!(result.result.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.id, Some(json!("test_request_123")));
        
        let result_value = result.result.unwrap();
        assert_eq!(result_value["valid"], true);
        assert_eq!(result_value["share_id"], "test_share_id");
        assert_eq!(result_value["miner_reputation"], 0.85);
        assert_eq!(result_value["difficulty_achieved"], 1.5);
        assert_eq!(result_value["pool_signature"], "test_signature");
    }

    #[test]
    fn test_create_pool_validation_response_with_error() {
        let response = create_test_pool_validation_response(false, Some("Validation failed".to_string()));
        let request_id = "test_request_456";
        
        let result = MiningPoolResponseHandler::create_pool_validation_response(&response, request_id);
        
        assert!(result.result.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.id, Some(json!("test_request_456")));
        
        let result_value = result.result.unwrap();
        assert_eq!(result_value["valid"], false);
        assert_eq!(result_value["error"], "Validation failed");
    }

    #[test]
    fn test_handle_successful_validation() {
        let response = create_test_pool_validation_response(true, None);
        let request_id = "test_request_789";
        let context_request_id = "context_123";
        
        let result = MiningPoolResponseHandler::handle_successful_validation(
            &response,
            request_id,
            context_request_id,
        );
        
        assert!(result.result.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.id, Some(json!("test_request_789")));
        
        let result_value = result.result.unwrap();
        assert_eq!(result_value["valid"], true);
        assert_eq!(result_value["share_id"], "test_share_id");
    }

    #[test]
    fn test_handle_failed_validation() {
        let error = AppError::Validation("Share validation failed".to_string());
        let request_id = "test_request_999";
        let context_request_id = "context_456";
        
        let result = MiningPoolResponseHandler::handle_failed_validation(
            &error,
            request_id,
            context_request_id,
        );
        
        assert!(result.result.is_none());
        assert!(result.error.is_some());
        assert_eq!(result.id, Some(json!("test_request_999")));
        
        let error_value = result.error.unwrap();
        assert_eq!(error_value.code, -32603); // Internal error code
        assert!(error_value.message.contains("Share validation failed"));
    }

    #[test]
    fn test_handle_failed_validation_with_different_error_types() {
        let errors = vec![
            AppError::Validation("Validation error".to_string()),
            AppError::RateLimit,
            AppError::Authentication("Auth failed".to_string()),
        ];
        
        for error in errors {
            let result = MiningPoolResponseHandler::handle_failed_validation(
                &error,
                "test_id",
                "context_id",
            );
            
            assert!(result.result.is_none());
            assert!(result.error.is_some());
            assert_eq!(result.id, Some(json!("test_id")));
        }
    }

    #[test]
    fn test_validate_pool_response_valid() {
        let response = create_test_pool_validation_response(true, None);
        
        let result = MiningPoolResponseHandler::validate_pool_response(&response);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pool_response_invalid() {
        let response = create_test_pool_validation_response(false, Some("Custom error message".to_string()));
        
        let result = MiningPoolResponseHandler::validate_pool_response(&response);
        
        assert!(result.is_err());
        if let Err(AppError::Validation(message)) = result {
            assert_eq!(message, "Custom error message");
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_validate_pool_response_invalid_without_error() {
        let response = create_test_pool_validation_response(false, None);
        
        let result = MiningPoolResponseHandler::validate_pool_response(&response);
        
        assert!(result.is_err());
        if let Err(AppError::Validation(message)) = result {
            assert_eq!(message, "Pool share validation failed");
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_create_pool_validation_response_with_empty_request_id() {
        let response = create_test_pool_validation_response(true, None);
        let request_id = "";
        
        let result = MiningPoolResponseHandler::create_pool_validation_response(&response, request_id);
        
        assert!(result.result.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.id, Some(json!("")));
    }

    #[test]
    fn test_create_pool_validation_response_with_special_characters() {
        let response = create_test_pool_validation_response(true, None);
        let request_id = "test_request_with_special_chars_!@#$%^&*()";
        
        let result = MiningPoolResponseHandler::create_pool_validation_response(&response, request_id);
        
        assert!(result.result.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.id, Some(json!("test_request_with_special_chars_!@#$%^&*()")));
    }

    #[test]
    fn test_create_pool_validation_response_with_unicode_characters() {
        let response = create_test_pool_validation_response(true, None);
        let request_id = "test_request_with_unicode_🚀🌍🎯";
        
        let result = MiningPoolResponseHandler::create_pool_validation_response(&response, request_id);
        
        assert!(result.result.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.id, Some(json!("test_request_with_unicode_🚀🌍🎯")));
    }

    #[test]
    fn test_handle_successful_validation_with_none_values() {
        let response = PoolValidationResponse {
            valid: true,
            share_id: None,
            pool_signature: None,
            difficulty_achieved: None,
            miner_reputation: None,
            timestamp: chrono::Utc::now(),
            error: None,
        };
        let request_id = "test_request_none";
        let context_request_id = "context_none";
        
        let result = MiningPoolResponseHandler::handle_successful_validation(
            &response,
            request_id,
            context_request_id,
        );
        
        assert!(result.result.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.id, Some(json!("test_request_none")));
        
        let result_value = result.result.unwrap();
        assert_eq!(result_value["valid"], true);
        // Note: serde_json serializes None values as null, not missing fields
        assert_eq!(result_value["share_id"], serde_json::Value::Null);
        assert_eq!(result_value["miner_reputation"], serde_json::Value::Null);
        assert_eq!(result_value["difficulty_achieved"], serde_json::Value::Null);
        assert_eq!(result_value["pool_signature"], serde_json::Value::Null);
    }

    #[test]
    fn test_handle_failed_validation_with_empty_strings() {
        let error = AppError::Validation("".to_string());
        let request_id = "";
        let context_request_id = "";
        
        let result = MiningPoolResponseHandler::handle_failed_validation(
            &error,
            request_id,
            context_request_id,
        );
        
        assert!(result.result.is_none());
        assert!(result.error.is_some());
        assert_eq!(result.id, Some(json!("")));
        
        let error_value = result.error.unwrap();
        assert_eq!(error_value.code, -32603);
        assert!(error_value.message.contains("Validation error"));
    }
}
