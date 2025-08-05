use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    models::{JsonRpcRequest, JsonRpcResponse, RequestContext},
    validation::MethodValidator,
};
use std::sync::Arc;
use tracing::instrument;

/// RPC request handler
pub struct RpcHandler {
    validator: Arc<MethodValidator>,
}

impl RpcHandler {
    /// Create a new RPC handler
    pub async fn new(_config: &AppConfig) -> AppResult<Self> {
        let validator = Arc::new(MethodValidator::new());

        Ok(Self {
            validator,
        })
    }

    /// Handle an RPC request
    #[instrument(skip(self))]
    pub async fn handle_request(
        &self,
        request: JsonRpcRequest,
        _context: RequestContext,
    ) -> AppResult<JsonRpcResponse> {
        // Validate method and parameters
        self.validator.validate_method(&request.method, &request.params)?;

        // For now, return a mock response
        let result = serde_json::json!({
            "result": "mock_response",
            "method": request.method,
            "params": request.params
        });

        Ok(JsonRpcResponse::success(result, request.id))
    }
} 