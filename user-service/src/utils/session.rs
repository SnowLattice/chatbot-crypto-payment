use std::fmt::Debug;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use sea_orm::TransactionTrait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    client::redis::{RedisClient, RedisClientExt},
    entity::session,
    repositories, ServiceState,
};

pub trait RedisKey: Debug + Display {
    type Value: Serialize + DeserializeOwned + Debug;
    const EXPIRE_TIME: Duration;
    fn expire(&self) -> Duration {
        Self::EXPIRE_TIME
    }
}

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct SessionKey {
    pub user_id: i64,
}

impl RedisKey for SessionKey {
    type Value = session::Model;
    const EXPIRE_TIME: Duration = Duration::from_secs(600);
}

impl Display for SessionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SESSION_KEY_{}", self.user_id)
    }
}

pub async fn set<K>(client: &RedisClient, (key, value): (&K, &K::Value)) -> Result<(), String>
where
    K: RedisKey,
{
    info!("Set value to redis key :{key:?} value :{value:?}");
    let value =
        serde_json::to_string(value).map_err(|e| format!("serde to_string error: {}", e))?;
    client
        .set(&key.to_string(), &value, K::EXPIRE_TIME)
        .await
        .map_err(|e| format!("Redis client set error: {}", e))?;
    Ok(())
}

pub async fn get<K>(client: &RedisClient, key: &K) -> Result<Option<K::Value>, String>
where
    K: RedisKey,
{
    Ok(client
        .get(&key.to_string())
        .await
        .map_err(|e| format!("Redis client get error: {}", e))?
        .map(|v| serde_json::from_str::<K::Value>(&v))
        .transpose()
        .map_err(|e| format!("Redis transpose error: {}", e))?)
}
pub async fn del(client: &RedisClient, key: &impl RedisKey) -> Result<bool, String> {
    client
        .del(&key.to_string())
        .await
        .map_err(|e| format!("Redis client del error: {}", e))
}

pub async fn check_exist_key(redis: &RedisClient, key: &impl RedisKey) -> Result<bool, String> {
    Ok(redis
        .exist(&key.to_string())
        .await
        .map_err(|e| format!("Redis client check existing error: {}", e))?)
}

pub async fn get_session_by_user_id(
    state: Arc<ServiceState>,
    user_id: i64,
) -> Result<session::Model, String> {
    let transaction = state
        .db
        .begin()
        .await
        .map_err(|e| format!("Failed to start a database transaction: {}", e))?;
    let session_key = SessionKey { user_id: user_id };
    match get(&state.redis, &session_key)
        .await
        .map_err(|e| format!("Failed to get session from Redis: {}", e))?
    {
        Some(model) => Ok(model),
        None => {
            let session_data = repositories::session::find_by_user_id(&transaction, user_id)
                .await
                .map_err(|e| format!("Failed to find session by ID: {}", e))?;

            if session_data.is_none() {
                return Err("Session data not found".to_string());
            }
            transaction
                .commit()
                .await
                .map_err(|e| format!("Failed to commit transaction: {}", e))?;
            let session_data = session_data.unwrap();
            set(&state.redis, (&session_key, &session_data))
                .await
                .map_err(|e| format!("Failed to set session in Redis: {}", e))?;

            return Ok(session_data);
        }
    }
}
