use std::fmt::Debug;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    client::redis::{RedisClient, RedisClientExt},
    ServiceState,
};

pub trait RedisKey: Debug + Display {
    type Value: Serialize + DeserializeOwned + Debug;
    const EXPIRE_TIME: Duration;
    fn expire(&self) -> Duration {
        Self::EXPIRE_TIME
    }
}

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct SubscriptionKey {
    pub user_id: i64,
}

impl RedisKey for SubscriptionKey {
    type Value = String;
    const EXPIRE_TIME: Duration = Duration::from_secs(600);
}

impl Display for SubscriptionKey {
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
