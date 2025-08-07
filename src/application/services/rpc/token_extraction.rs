//! Auth token extraction helper for RpcService

use crate::domain::rpc::RpcRequest;

pub fn extract_bearer_token_from_request(_request: &RpcRequest) -> Option<String> {
    // Token is now extracted at the HTTP layer from the Authorization header
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_request(ua: Option<&str>) -> RpcRequest {
        RpcRequest {
            method: "m".to_string(),
            parameters: Some(json!([])),
            id: Some(json!(1)),
            client_info: crate::domain::rpc::ClientInfo {
                ip_address: "127.0.0.1".to_string(),
                user_agent: ua.map(|s| s.to_string()),
                auth_token: None,
                timestamp: chrono::Utc::now(),
            },
        }
    }

    #[test]
    fn extract_token_ok() {
        let req = make_request(Some("Some UA"));
        let token = extract_bearer_token_from_request(&req);
        assert!(token.is_none());
    }

    #[test]
    fn extract_token_none() {
        let req = make_request(Some("Some UA"));
        let token = extract_bearer_token_from_request(&req);
        assert!(token.is_none());
    }
}


