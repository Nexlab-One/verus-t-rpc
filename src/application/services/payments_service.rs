//! Payment service orchestrating quotes, submission, verification, and token issuance

use std::sync::Arc;

use crate::config::AppConfig;
use crate::domain::payments::{PaymentSession, PaymentStatus, PaymentTier, ShieldedAddressType};
use crate::domain::rpc::{ClientInfo, RpcRequest};
use crate::infrastructure::adapters::{ExternalRpcAdapter, PaymentsStore, TokenIssuerAdapter, TokenIssuanceMode, TokenIssuanceRequest, RevocationStore};
use crate::shared::error::{AppError, AppResult};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use jsonwebtoken::{DecodingKey, Validation, Algorithm, decode};
use crate::infrastructure::adapters::token_issuer::JwtClaims;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentsConfig {
    pub enabled: bool,
    pub address_types: Vec<ShieldedAddressType>,
    pub default_address_type: ShieldedAddressType,
    pub min_confirmations: u32,
    pub session_ttl_minutes: u32,
    pub tiers: Vec<PaymentTier>,
    pub require_viewing_key: bool,
}

impl Default for PaymentsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            address_types: vec![ShieldedAddressType::Orchard, ShieldedAddressType::Sapling],
            default_address_type: ShieldedAddressType::Orchard,
            min_confirmations: 1,
            session_ttl_minutes: 30,
            tiers: vec![
                PaymentTier { id: "basic".to_string(), amount_vrsc: 1.0, description: Some("Basic access".to_string()), permissions: vec!["read".to_string()] },
                PaymentTier { id: "pro".to_string(), amount_vrsc: 5.0, description: Some("Pro access".to_string()), permissions: vec!["read".to_string(), "write".to_string()] },
            ],
            require_viewing_key: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentQuoteRequest {
    pub tier_id: String,
    pub address_type: Option<ShieldedAddressType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentQuoteResponse {
    pub payment_id: String,
    pub tier_id: String,
    pub amount_vrsc: f64,
    pub address: String,
    pub address_type: ShieldedAddressType,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSubmitRequest {
    pub payment_id: String,
    pub rawtx_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSubmitResponse {
    pub txid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentStatusResponse {
    pub status: PaymentStatus,
    pub confirmations: u32,
    pub amount_vrsc: f64,
    pub address: String,
    pub txid: Option<String>,
    pub provisional_token: Option<String>,
    pub final_token: Option<String>,
}

pub struct PaymentsService {
    config: Arc<AppConfig>,
    payments_config: PaymentsConfig,
    rpc: Arc<ExternalRpcAdapter>,
    store: Arc<PaymentsStore>,
    token_issuer: Arc<TokenIssuerAdapter>,
    revocations: Arc<RevocationStore>,
}

impl PaymentsService {
    /// Refresh in-memory payments configuration from the application configuration
    pub fn refresh_from_app_config(&mut self) {
        let p = &self.config.payments;
        self.payments_config.enabled = p.enabled;
        self.payments_config.address_types = p.address_types.iter().filter_map(|s| s.parse().ok()).collect();
        self.payments_config.default_address_type = p.default_address_type.parse().unwrap_or(ShieldedAddressType::Orchard);
        self.payments_config.min_confirmations = p.min_confirmations;
        self.payments_config.session_ttl_minutes = p.session_ttl_minutes as u32;
        self.payments_config.require_viewing_key = p.require_viewing_key;
        self.payments_config.tiers = p.tiers.iter().map(|t| PaymentTier {
            id: t.id.clone(),
            amount_vrsc: t.amount_vrsc,
            description: t.description.clone(),
            permissions: t.permissions.clone(),
        }).collect();
    }
    pub fn new(
        config: Arc<AppConfig>,
        payments_config: PaymentsConfig,
        rpc: Arc<ExternalRpcAdapter>,
        store: Arc<PaymentsStore>,
        token_issuer: Arc<TokenIssuerAdapter>,
        revocations: Arc<RevocationStore>,
    ) -> Self {
        // Always refresh from AppConfig to ensure runtime config is applied
        let mut svc = Self { config, payments_config, rpc, store, token_issuer, revocations };
        svc.refresh_from_app_config();
        svc
    }

    fn find_tier(&self, id: &str) -> Option<PaymentTier> {
        self.payments_config.tiers.iter().find(|t| t.id == id).cloned()
    }

    pub async fn create_quote(
        &self,
        req: PaymentQuoteRequest,
        client_info: &ClientInfo,
    ) -> AppResult<PaymentQuoteResponse> {
        if !self.payments_config.enabled { return Err(AppError::Security("payments disabled".into())); }

        let tier = self
            .find_tier(&req.tier_id)
            .ok_or_else(|| AppError::Validation("unknown tier".into()))?;

        let addr_type = req.address_type.unwrap_or(self.payments_config.default_address_type.clone());
        if !self.payments_config.address_types.contains(&addr_type) {
            return Err(AppError::Validation("unsupported address type".into()));
        }

        // If viewing-key-only mode is required, avoid creating a new address.
        // Instead, select a compatible existing shielded address from the wallet.
        let address = if self.payments_config.require_viewing_key {
            if self.config.payments.viewing_keys.is_empty() {
                return Err(AppError::Security("Viewing key required but not configured".into()));
            }

            // List available z-addresses
            let list_req = RpcRequest::new(
                "z_listaddresses".to_string(),
                Some(serde_json::Value::Array(vec![])),
                Some(json!(Uuid::new_v4().to_string())),
                client_info.clone(),
            );
            let list_res = self.rpc.send_request(&list_req).await?;
            let candidates: Vec<String> = list_res
                .result
                .and_then(|v| v.as_array().cloned())
                .ok_or_else(|| AppError::Rpc("z_listaddresses returned invalid result".into()))?
                .into_iter()
                .filter_map(|val| val.as_str().map(|s| s.to_string()))
                .collect();

            if candidates.is_empty() {
                return Err(AppError::Security("No shielded addresses available under viewing keys".into()));
            }

            // Find an address matching the requested type via z_validateaddress
            let mut selected: Option<String> = None;
            for addr in candidates {
                let validate_req = RpcRequest::new(
                    "z_validateaddress".to_string(),
                    Some(serde_json::Value::Array(vec![serde_json::Value::String(addr.clone())])),
                    Some(json!(Uuid::new_v4().to_string())),
                    client_info.clone(),
                );
                if let Ok(val_res) = self.rpc.send_request(&validate_req).await {
                    if let Some(obj) = val_res.result.and_then(|v| v.as_object().cloned()) {
                        let addr_type_str = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        if addr_type_str.eq_ignore_ascii_case(addr_type.as_str()) {
                            selected = Some(addr);
                            break;
                        }
                    }
                }
            }

            selected.ok_or_else(|| AppError::Security("No compatible shielded address for requested type".into()))?
        } else {
            // Ask daemon for a new shielded address (z_getnewaddress "orchard" | "sapling")
            let method = "z_getnewaddress".to_string();
            let params = serde_json::Value::Array(vec![serde_json::Value::String(addr_type.as_str().to_string())]);
            let rpc_req = RpcRequest::new(method, Some(params), Some(json!(Uuid::new_v4().to_string())), client_info.clone());
            let rpc_res = self.rpc.send_request(&rpc_req).await?;
            rpc_res
                .result
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .ok_or_else(|| AppError::Rpc("invalid z_getnewaddress result".into()))?
        };

        let now = Utc::now();
        let expires_at = now + Duration::minutes(self.payments_config.session_ttl_minutes as i64);
        let payment_id = Uuid::new_v4().to_string();

        let session = PaymentSession {
            payment_id: payment_id.clone(),
            tier_id: tier.id.clone(),
            address: address.clone(),
            address_type: addr_type.clone(),
            amount_vrsc: tier.amount_vrsc,
            created_at: now,
            expires_at,
            client_ip: Some(client_info.ip_address.clone()),
            user_agent: client_info.user_agent.clone(),
            status: PaymentStatus::Pending,
            txid: None,
            confirmations: 0,
            provisional_token: None,
            final_token: None,
        };
        self.store.put(&session).await?;

        Ok(PaymentQuoteResponse {
            payment_id,
            tier_id: tier.id,
            amount_vrsc: tier.amount_vrsc,
            address,
            address_type: addr_type,
            expires_at,
        })
    }

    pub async fn submit_raw_transaction(&self, req: PaymentSubmitRequest, client_info: &ClientInfo) -> AppResult<PaymentSubmitResponse> {
        let mut session = self
            .store
            .get(&req.payment_id)
            .await?
            .ok_or_else(|| AppError::Validation("unknown payment_id".into()))?;

        if session.is_expired() { return Err(AppError::Validation("payment session expired".into())); }
        if session.status != PaymentStatus::Pending && session.status != PaymentStatus::Submitted {
            return Err(AppError::Validation("invalid state for submission".into()));
        }

        // Basic sanity checks on hex
        if req.rawtx_hex.len() < 100 || !req.rawtx_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AppError::Validation("invalid raw tx hex".into()));
        }

        // Broadcast raw tx
        let rpc_req = RpcRequest::new(
            "sendrawtransaction".to_string(),
            Some(serde_json::Value::Array(vec![serde_json::Value::String(req.rawtx_hex)])),
            Some(json!(Uuid::new_v4().to_string())),
            client_info.clone(),
        );
        let rpc_res = self.rpc.send_request(&rpc_req).await?;
        let txid = rpc_res
            .result
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| AppError::Rpc("invalid sendrawtransaction result".into()))?;

        session.txid = Some(txid.clone());
        session.status = PaymentStatus::Submitted;
        self.store.put(&session).await?;

        Ok(PaymentSubmitResponse { txid })
    }

    pub async fn check_status(&self, payment_id: &str, client_info: &ClientInfo) -> AppResult<PaymentStatusResponse> {
        let mut session = self
            .store
            .get(payment_id)
            .await?
            .ok_or_else(|| AppError::Validation("unknown payment_id".into()))?;

        if session.is_expired() && session.status != PaymentStatus::Finalized {
            // If we had issued a provisional token, revoke it
            if let Some(token) = &session.provisional_token {
                let _ = self.revoke_token_by_string(token).await;
            }
            session.status = PaymentStatus::Expired;
            self.store.put(&session).await?;
        }

        // If we have a txid, verify receipt via z_viewtransaction
        if let Some(txid) = session.txid.clone() {
            // z_viewtransaction requires the wallet to have a viewing/spending key for the outputs
            let rpc_req = RpcRequest::new(
                "z_viewtransaction".to_string(),
                Some(serde_json::Value::Array(vec![serde_json::Value::String(txid.clone())])),
                Some(json!(Uuid::new_v4().to_string())),
                client_info.clone(),
            );
            let rpc_res = self.rpc.send_request(&rpc_req).await?;

            // We expect a structure containing received outputs; we conservatively search JSON
            let mut paid_amount = 0.0f64;
            let mut matched = false;
            if let Some(v) = rpc_res.result {
                // Look for any output to our session.address and sum amounts
                if let Some(outputs) = v.get("outputs").and_then(|o| o.as_array()) {
                    for o in outputs {
                        let addr_ok = o.get("address").and_then(|a| a.as_str()) == Some(session.address.as_str());
                        let amt = o.get("amount").and_then(|a| a.as_f64()).unwrap_or(0.0);
                        if addr_ok {
                            paid_amount += amt;
                            matched = true;
                        }
                    }
                }
            }

            if matched && paid_amount + 1e-12 >= session.amount_vrsc {
                // Query confirmations via getrawtransaction verbose=true or gettransaction
                // Fallback: use getrawtransaction <txid> 1 (verbose) for confirmations
                let raw_req = RpcRequest::new(
                    "getrawtransaction".to_string(),
                    Some(serde_json::Value::Array(vec![serde_json::Value::String(txid.clone()), serde_json::Value::Number(1u64.into())])),
                    Some(json!(Uuid::new_v4().to_string())),
                    client_info.clone(),
                );
                let raw_res = self.rpc.send_request(&raw_req).await?;
                let confirmations = raw_res
                    .result
                    .and_then(|r| r.get("confirmations").and_then(|c| c.as_u64()))
                    .unwrap_or(0) as u32;
                session.confirmations = confirmations;

                // Issue provisional token at 1 conf if configured; then replace once finalized
                if confirmations >= self.payments_config.min_confirmations {
                    if session.provisional_token.is_none() {
                        let token = self.issue_token(&session, true, client_info).await?;
                        session.provisional_token = Some(token);
                        session.status = PaymentStatus::Confirmed1;
                    }
                } else {
                    session.status = PaymentStatus::Verified;
                }

                // Optional second-check/finalization when deeper confirmations available (e.g., >=2)
                if confirmations >= (self.payments_config.min_confirmations.max(2)) {
                    if session.final_token.is_none() {
                        let token = self.issue_token(&session, false, client_info).await?;
                        session.final_token = Some(token);
                        session.status = PaymentStatus::Finalized;
                    }
                }

                self.store.put(&session).await?;
            } else if session.provisional_token.is_some() {
                // If we can no longer validate recipient match but had issued a provisional token, revoke it
                // Note: this requires the Authentication layer to check revocations; handled via RevocationStore
                if let Some(token) = &session.provisional_token {
                    let _ = self.revoke_token_by_string(token).await;
                }
                session.provisional_token = None;
                session.status = PaymentStatus::Failed;
                self.store.put(&session).await?;
            }
        }

        Ok(PaymentStatusResponse {
            status: session.status.clone(),
            confirmations: session.confirmations,
            amount_vrsc: session.amount_vrsc,
            address: session.address.clone(),
            txid: session.txid.clone(),
            provisional_token: session.provisional_token.clone(),
            final_token: session.final_token.clone(),
        })
    }

    async fn issue_token(&self, session: &PaymentSession, provisional: bool, client_info: &ClientInfo) -> AppResult<String> {
        let tier = self
            .find_tier(&session.tier_id)
            .ok_or_else(|| AppError::Internal("tier not found".into()))?;

        let mut permissions = tier.permissions.clone();
        if provisional {
            // Mark token as provisional (lower privileges)
            permissions.push("provisional".to_string());
        } else {
            permissions.push("paid".to_string());
        }

        let req = TokenIssuanceRequest {
            user_id: format!("pay_{}", session.payment_id),
            permissions,
            custom_expiration: None,
            client_ip: session.client_ip.clone().or_else(|| Some(client_info.ip_address.clone())),
            user_agent: session.user_agent.clone(),
            mode: TokenIssuanceMode::Anonymous,
            pow_challenge: None,
        };
        let token_res = self.token_issuer.issue_token(req).await?;
        Ok(token_res.token)
    }

    async fn revoke_token_by_string(&self, token: &str) -> AppResult<()> {
        // Decode to extract jti and exp
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[&self.config.security.jwt.audience]);
        validation.set_issuer(&[&self.config.security.jwt.issuer]);
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.config.security.jwt.secret_key.as_ref()),
            &validation,
        ).map_err(|e| AppError::Authentication(format!("JWT decode failed: {}", e)))?;
        let claims = token_data.claims;
        let now = Utc::now().timestamp() as u64;
        let ttl = if (claims.exp as u64) > now { (claims.exp as u64) - now } else { 0 };
        // Revoke with remaining TTL (fallback to 1h if expired)
        let ttl = if ttl == 0 { 3600 } else { ttl };
        self.revocations.revoke(&claims.jti, ttl).await.map_err(|e| AppError::Internal(format!("revocation failed: {}", e)))
    }
}


