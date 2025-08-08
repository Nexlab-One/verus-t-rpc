//! Redis-backed payments store

use crate::shared::error::{AppError, AppResult};
use crate::domain::payments::PaymentSession;
use redis::{aio::ConnectionManager, AsyncCommands};
use std::sync::Arc;

/// Abstraction for persisting payment sessions
#[derive(Clone)]
pub struct PaymentsStore {
    redis: Option<Arc<ConnectionManager>>, // optional; can operate in-memory only if None
    memory: Arc<tokio::sync::RwLock<std::collections::HashMap<String, PaymentSession>>>,
}

impl PaymentsStore {
    pub fn new(redis: Option<Arc<ConnectionManager>>) -> Self {
        Self {
            redis,
            memory: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    fn key(payment_id: &str) -> String {
        format!("payments:{}", payment_id)
    }

    pub async fn put(&self, session: &PaymentSession) -> AppResult<()> {
        let serialized = serde_json::to_vec(session)
            .map_err(|e| AppError::Internal(format!("serialize payment: {}", e)))?;

        if let Some(redis) = &self.redis {
            let mut conn = (**redis).clone();
            let key = Self::key(&session.payment_id);
            // TTL: keep for 48h
            let _: () = conn
                .set_ex(key, serialized, 48 * 3600)
                .await
                .map_err(|e| AppError::Internal(format!("redis set: {}", e)))?;
        }

        // Always mirror to memory
        self.memory.write().await.insert(session.payment_id.clone(), session.clone());
        Ok(())
    }

    pub async fn get(&self, payment_id: &str) -> AppResult<Option<PaymentSession>> {
        if let Some(redis) = &self.redis {
            let mut conn = (**redis).clone();
            let key = Self::key(payment_id);
            let data: Option<Vec<u8>> = conn
                .get(key)
                .await
                .map_err(|e| AppError::Internal(format!("redis get: {}", e)))?;
            if let Some(bytes) = data {
                let session: PaymentSession = serde_json::from_slice(&bytes)
                    .map_err(|e| AppError::Internal(format!("deserialize payment: {}", e)))?;
                // mirror to memory
                self.memory.write().await.insert(payment_id.to_string(), session.clone());
                return Ok(Some(session));
            }
        }
        Ok(self.memory.read().await.get(payment_id).cloned())
    }
}


