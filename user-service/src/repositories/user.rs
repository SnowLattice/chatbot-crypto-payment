use crate::entity::user;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

#[tracing::instrument(skip_all)]
pub async fn save(tx: &DatabaseTransaction, user_id: i64) -> Result<Uuid, String> {
    let new_user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        total_credits: Set(15),
        credits_remaining: Set(15),
        subscription_status: Set(false),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    match new_user.insert(tx).await {
        Ok(user) => Ok(user.id),
        Err(e) => Err(format!("New user record is not saved successfully: {}", e)),
    }
}

#[tracing::instrument(skip_all)]
pub async fn find_by_id(tx: &DatabaseTransaction, id: Uuid) -> Result<Option<user::Model>, String> {
    match user::Entity::find_by_id(id).one(tx).await {
        Ok(Some(model)) => Ok(Some(model)),
        Ok(None) => Err(format!("No user record found with id: {}", id)),
        Err(e) => Err(format!("Error finding user by id: {}", e)),
    }
}

#[tracing::instrument(skip_all)]
pub async fn find_by_user_id(
    tx: &DatabaseTransaction,
    user_id: i64,
) -> Result<Option<user::Model>, String> {
    match user::Entity::find()
        .filter(user::Column::UserId.eq(user_id))
        .one(tx)
        .await
    {
        Ok(model) => Ok(model),
        Err(e) => Err(format!("Error finding user by user_id: {}", e)),
    }
}

#[tracing::instrument(skip_all)]
pub async fn exist_by_user_id(tx: &DatabaseTransaction, user_id: i64) -> Result<bool, String> {
    match user::Entity::find()
        .filter(user::Column::UserId.eq(user_id))
        .one(tx)
        .await
    {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(format!("Error checking existence by user_id: {}", e)),
    }
}
