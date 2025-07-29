use crate::entity::session;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use serde_json::json;
use uuid::Uuid;

#[tracing::instrument(skip_all)]
pub async fn save(tx: &DatabaseTransaction, user_id: i64) -> Result<Uuid, String> {
    let new_session = session::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        credits_remaining: Set(15),
        subscription_status: Set(false),
        last_active_timestamp: Set(Utc::now().timestamp()),
        preferences: Set(json!({
            "default_mode": "GPT-4o",
            "notifications": true
        })),
        session_metadata: Set(json!({
            "last_mode_used": "GPT-4o",
            "recent_actions": ["request_made"]
        })),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    match new_session.insert(tx).await {
        Ok(session) => Ok(session.id),
        Err(e) => Err(format!("Session record was not saved successfully: {}", e)),
    }
}

#[tracing::instrument(skip_all)]
pub async fn find_by_id(
    tx: &DatabaseTransaction,
    id: Uuid,
) -> Result<Option<session::Model>, String> {
    match session::Entity::find_by_id(id).one(tx).await {
        Ok(Some(model)) => Ok(Some(model)),
        Ok(None) => Err(format!("No session record found with id: {}", id)),
        Err(e) => Err(format!("Error finding session by id: {}", e)),
    }
}

#[tracing::instrument(skip_all)]
pub async fn find_by_user_id(
    tx: &DatabaseTransaction,
    user_id: i64,
) -> Result<Option<session::Model>, String> {
    match session::Entity::find()
        .filter(session::Column::UserId.eq(user_id))
        .one(tx)
        .await
    {
        Ok(model) => Ok(model),
        Err(e) => Err(format!("Error finding session by session_id: {}", e)),
    }
}

#[tracing::instrument(skip_all)]
pub async fn exist_by_user_id(tx: &DatabaseTransaction, user_id: i64) -> Result<bool, String> {
    match session::Entity::find()
        .filter(session::Column::UserId.eq(user_id))
        .one(tx)
        .await
    {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(format!("Error checking existence by user_id: {}", e)),
    }
}
