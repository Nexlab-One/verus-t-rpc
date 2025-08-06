//! Converters between domain and infrastructure models

use crate::{
    domain::{rpc::*, security::*},
    infrastructure::http::models::{JsonRpcRequest, JsonRpcResponse, JsonRpcError, RequestContext},
    shared::error::AppResult,
};
use serde_json::Value;

/// Converter for transforming between domain and infrastructure models
pub struct ModelConverter;

impl ModelConverter {
    /// Convert infrastructure request to domain request
    pub fn to_domain_request(
        infra_request: &JsonRpcRequest,
        context: &RequestContext,
    ) -> AppResult<RpcRequest> {
        let client_info = ClientInfo {
            ip_address: context.client_ip.clone(),
            user_agent: context.user_agent.clone(),
            timestamp: context.timestamp,
        };

        Ok(RpcRequest::new(
            infra_request.method.clone(),
            infra_request.params.clone(),
            infra_request.id.clone(),
            client_info,
        ))
    }

    /// Convert domain response to infrastructure response
    pub fn to_infrastructure_response(domain_response: &RpcResponse) -> JsonRpcResponse {
        match &domain_response.error {
            Some(error) => {
                let infra_error = JsonRpcError::new(
                    error.code,
                    error.message.clone(),
                    error.data.clone(),
                );
                JsonRpcResponse::error(infra_error, domain_response.id.clone())
            }
            None => {
                let result = domain_response.result.clone().unwrap_or(Value::Null);
                JsonRpcResponse::success(result, domain_response.id.clone())
            }
        }
    }

    /// Convert domain error to infrastructure error
    pub fn to_infrastructure_error(domain_error: &RpcError) -> JsonRpcError {
        JsonRpcError::new(
            domain_error.code,
            domain_error.message.clone(),
            domain_error.data.clone(),
        )
    }

    /// Convert infrastructure error to domain error
    pub fn to_domain_error(infra_error: &JsonRpcError) -> RpcError {
        RpcError::new(
            infra_error.code,
            infra_error.message.clone(),
            infra_error.data.clone(),
            None, // No context for infrastructure errors
        )
    }

    /// Create security context from request context
    pub fn to_security_context(
        request_context: &RequestContext,
        auth_token: Option<String>,
        user_permissions: Vec<String>,
        development_mode: bool,
    ) -> SecurityContext {
        SecurityContext {
            client_ip: request_context.client_ip.clone(),
            user_agent: request_context.user_agent.clone(),
            auth_token,
            user_permissions,
            timestamp: request_context.timestamp,
            request_id: request_context.request_id.clone(),
            development_mode,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_to_domain_request() {
        let infra_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "getinfo".to_string(),
            params: Some(serde_json::json!([])),
            id: Some(serde_json::json!(1)),
        };

        let context = RequestContext {
            request_id: "test-123".to_string(),
            client_ip: "127.0.0.1".to_string(),
            user_agent: Some("test-agent".to_string()),
            timestamp: Utc::now(),
            method: "getinfo".to_string(),
            params: Some(serde_json::json!([])),
        };

        let result = ModelConverter::to_domain_request(&infra_request, &context);
        assert!(result.is_ok());

        let domain_request = result.unwrap();
        assert_eq!(domain_request.method, "getinfo");
        assert_eq!(domain_request.client_info.ip_address, "127.0.0.1");
        assert_eq!(domain_request.client_info.user_agent, Some("test-agent".to_string()));
    }

    #[test]
    fn test_to_infrastructure_response_success() {
        let domain_response = RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"version": "1.0.0"})),
            error: None,
            id: Some(serde_json::json!(1)),
        };

        let infra_response = ModelConverter::to_infrastructure_response(&domain_response);
        assert!(infra_response.result.is_some());
        assert!(infra_response.error.is_none());
        assert_eq!(infra_response.result.unwrap(), serde_json::json!({"version": "1.0.0"}));
    }

    #[test]
    fn test_to_infrastructure_response_error() {
        let domain_error = RpcError::new(
            -32601,
            "Method not found".to_string(),
            None,
            None,
        );

        let domain_response = RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(domain_error),
            id: Some(serde_json::json!(1)),
        };

        let infra_response = ModelConverter::to_infrastructure_response(&domain_response);
        assert!(infra_response.result.is_none());
        assert!(infra_response.error.is_some());
        let error = infra_response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }

    #[test]
    fn test_to_infrastructure_response_null_result() {
        let domain_response = RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: None,
            id: Some(serde_json::json!(1)),
        };

        let infra_response = ModelConverter::to_infrastructure_response(&domain_response);
        assert!(infra_response.result.is_some());
        assert_eq!(infra_response.result.unwrap(), serde_json::Value::Null);
    }

    #[test]
    fn test_to_infrastructure_error() {
        let domain_error = RpcError::new(
            -32602,
            "Invalid params".to_string(),
            Some(serde_json::json!("error_data")),
            None,
        );

        let infra_error = ModelConverter::to_infrastructure_error(&domain_error);
        assert_eq!(infra_error.code, -32602);
        assert_eq!(infra_error.message, "Invalid params");
        assert_eq!(infra_error.data, Some(serde_json::json!("error_data")));
    }

    #[test]
    fn test_to_domain_error() {
        let infra_error = JsonRpcError::new(
            -32603,
            "Internal error".to_string(),
            Some(serde_json::json!("error_data")),
        );

        let domain_error = ModelConverter::to_domain_error(&infra_error);
        assert_eq!(domain_error.code, -32603);
        assert_eq!(domain_error.message, "Internal error");
        assert_eq!(domain_error.data, Some(serde_json::json!("error_data")));
        assert!(domain_error.context.is_none());
    }

    #[test]
    fn test_to_security_context() {
        let request_context = RequestContext {
            request_id: "req-456".to_string(),
            client_ip: "192.168.1.1".to_string(),
            user_agent: Some("test-browser".to_string()),
            timestamp: Utc::now(),
            method: "getinfo".to_string(),
            params: Some(serde_json::json!([])),
        };

        let auth_token = Some("jwt-token".to_string());
        let user_permissions = vec!["read".to_string(), "write".to_string()];
        let development_mode = true;

        let security_context = ModelConverter::to_security_context(
            &request_context,
            auth_token.clone(),
            user_permissions.clone(),
            development_mode,
        );

        assert_eq!(security_context.client_ip, "192.168.1.1");
        assert_eq!(security_context.user_agent, Some("test-browser".to_string()));
        assert_eq!(security_context.auth_token, auth_token);
        assert_eq!(security_context.user_permissions, user_permissions);
        assert_eq!(security_context.development_mode, development_mode);
        assert_eq!(security_context.request_id, "req-456");
    }
} 