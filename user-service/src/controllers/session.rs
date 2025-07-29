use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, Set, TransactionTrait};
use tracing::{error, info};

use crate::{
    dto::{request::SetSessionRequest, response::GetSessionResponse},
    entity, repositories,
    utils::{self, jwt::UserClaims, session::SessionKey},
    ServiceState,
};

pub async fn set_session(
    State(state): State<Arc<ServiceState>>,
    Json(req): Json<SetSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!(
        "Received 'set_session' request for user ID: {}",
        req.user_id
    );

    let transaction = state.db.begin().await.map_err(|e| {
        let error_message = format!(
            "Failed to start a database transaction for user ID {}: {}",
            req.user_id, e
        );
        error!("{}", error_message);
        (StatusCode::INTERNAL_SERVER_ERROR, error_message)
    })?;

    let session_data = utils::session::get_session_by_user_id(state.clone(), req.user_id)
        .await
        .map_err(|e| {
            let error_message = format!(
                "Failed to retrieve session data for user ID {}: {}",
                req.user_id, e
            );
            error!("{}", error_message);
            (StatusCode::INTERNAL_SERVER_ERROR, error_message)
        })?;

    let updated_model = entity::session::ActiveModel {
        id: Set(session_data.id),
        user_id: Set(session_data.user_id),
        subscription_status: Set({
            if req.subscription_status.is_some() {
                req.subscription_status.unwrap()
            } else if req.credits_remaining.is_some() && req.credits_remaining.unwrap() == 0 {
                false
            } else {
                session_data.subscription_status
            }
        }),
        credits_remaining: Set({
            if req.credits_remaining.is_some() {
                req.credits_remaining.unwrap()
            } else if req.subscription_status.is_some() && req.subscription_status.unwrap() == true {
                session_data.credits_remaining + state.config.charged_credit
            } else {
                session_data.credits_remaining
            }
        }),
        last_active_timestamp: Set(session_data.last_active_timestamp),
        preferences: Set({
            if req.preferences.is_some() {
                req.preferences.unwrap()
            } else {
                session_data.preferences
            }
        }),
        session_metadata: Set({
            if req.session_metadata.is_some() {
                req.session_metadata.unwrap()
            } else {
                session_data.session_metadata
            }
        }),
        created_at: Set(session_data.created_at),
        updated_at: Set(Utc::now()),
    };

    let updated_data: entity::session::Model =
        updated_model.update(&transaction).await.map_err(|e| {
            let error_message = format!(
                "Error updating session data for user ID {}: {}",
                req.user_id, e
            );
            error!("{}", error_message);
            (StatusCode::INTERNAL_SERVER_ERROR, error_message)
        })?;

    if session_data.credits_remaining != updated_data.credits_remaining
        || session_data.subscription_status != updated_data.subscription_status
    {
        let user_model = repositories::user::find_by_user_id(&transaction, req.user_id)
            .await
            .map_err(|e| {
                let error_message = format!(
                    "Error getting user model data by user ID {}: {}",
                    req.user_id, e
                );
                error!("{}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, error_message)
            })?;
        if user_model.is_none() {
            let error_message = format!("User record not found for user ID: {}", req.user_id);
            error!("{}", error_message);
            return Err((StatusCode::NOT_FOUND, error_message));
        }

        let user_model = user_model.unwrap();
        let mut updated_credit_remaining = user_model.credits_remaining;
        let mut updated_subscription_status = user_model.subscription_status;
        let mut updated_total_credits = user_model.total_credits;

        if session_data.subscription_status != updated_data.subscription_status {
            updated_total_credits += state.config.charged_credit;
            updated_subscription_status = true;
        }
        if session_data.credits_remaining != updated_data.credits_remaining {
            updated_credit_remaining = updated_data.credits_remaining;
        }

        let updated_user = entity::user::ActiveModel {
            id: Set(user_model.id),
            user_id: Set(user_model.user_id),
            total_credits: Set(updated_total_credits),
            credits_remaining: Set(updated_credit_remaining),
            subscription_status: Set(updated_subscription_status),
            created_at: Set(user_model.created_at),
            updated_at: Set(Utc::now()),
        };

        updated_user.update(&transaction).await.map_err(|e| {
            let error_message = format!(
                "Error updating user data for user ID {}: {}",
                req.user_id, e
            );
            error!("{}", error_message);
            (StatusCode::INTERNAL_SERVER_ERROR, error_message)
        })?;
    }

    let session_key = SessionKey {
        user_id: req.user_id,
    };
    utils::session::set(&state.redis, (&session_key, &updated_data))
        .await
        .map_err(|e| {
            let error_message = format!(
                "Failed to update session data in Redis for user ID {}: {}",
                req.user_id, e
            );
            error!("{}", error_message);
            (StatusCode::INTERNAL_SERVER_ERROR, error_message)
        })?;

    transaction.commit().await.map_err(|e| {
        let error_message = format!(
            "Failed to commit transaction for user ID {}: {}",
            req.user_id, e
        );
        error!("{}", error_message);
        (StatusCode::INTERNAL_SERVER_ERROR, error_message)
    })?;

    info!(
        "Successfully updated session data for user ID: {}",
        req.user_id
    );

    let response = Json(GetSessionResponse::default()).into_response();
    Ok(response)
}

pub async fn get_session(
    State(state): State<Arc<ServiceState>>,
    user: UserClaims,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("Received 'get_session' request for user ID: {}", user.uid);

    let session_model = utils::session::get_session_by_user_id(state.clone(), user.uid)
        .await
        .map_err(|e| {
            let error_message = format!(
                "Failed to retrieve session data for user ID {}: {}",
                user.uid, e
            );
            error!("{}", error_message);
            (StatusCode::INTERNAL_SERVER_ERROR, error_message)
        })?;

    info!(
        "Successfully retrieved session data for user ID: {}",
        user.uid
    );

    let response = Json(GetSessionResponse {
        subscription_status: session_model.subscription_status,
        credits_remaining: session_model.credits_remaining,
        preferences: session_model.preferences.clone(),
        session_metadata: session_model.session_metadata.clone(),
    })
    .into_response();
    Ok(response)
}
