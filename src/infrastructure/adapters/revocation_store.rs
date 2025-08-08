//! JWT revocation store (Redis-backed with memory fallback)

use std::sync::Arc;

use redis::{aio::ConnectionManager, AsyncCommands};

use crate::shared::error::{AppError, AppResult};

#[derive(Clone)]
pub struct RevocationStore {
    redis: Option<Arc<ConnectionManager>>, // optional
    memory: Arc<tokio::sync::RwLock<std::collections::HashSet<String>>>,
}

impl RevocationStore {
    pub fn new(redis: Option<Arc<ConnectionManager>>) -> Self {
        Self {
            redis,
            memory: Arc::new(tokio::sync::RwLock::new(std::collections::HashSet::new())),
        }
    }

    fn key(jti: &str) -> String { format!("jwt:revoked:{}", jti) }

    pub async fn revoke(&self, jti: &str, ttl_seconds: u64) -> AppResult<()> {
        if let Some(redis) = &self.redis {
            let mut conn = (**redis).clone();
            let _: () = conn
                .set_ex(Self::key(jti), 1u8, ttl_seconds)
                .await
                .map_err(|e| AppError::Internal(format!("redis set: {}", e)))?;
        }
        self.memory.write().await.insert(jti.to_string());
        Ok(())
    }

    pub async fn is_revoked(&self, jti: &str) -> AppResult<bool> {
        if let Some(redis) = &self.redis {
            let mut conn = (**redis).clone();
            let exists: bool = conn
                .exists(Self::key(jti))
                .await
                .map_err(|e| AppError::Internal(format!("redis exists: {}", e)))?;
            if exists { return Ok(true); }
        }
        Ok(self.memory.read().await.contains(jti))
    }
}


