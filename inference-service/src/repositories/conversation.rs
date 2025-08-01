use crate::entity::conversation::{self, Message, MessageType};
use chrono::Utc;
use rs_openai::chat::Role;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

pub async fn new_conversation(tx: &DatabaseTransaction, user_id: i64) -> Result<Uuid, String> {
    let new_conversation = conversation::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        conversation: Set(vec![]),
        title: Set(String::from("New Chat")),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    match new_conversation.insert(tx).await {
        Ok(user) => Ok(user.id),
        Err(e) => Err(format!(
            "New conversation record is not saved successfully: {}",
            e
        )),
    }
}

pub async fn find_by_user_id(
    tx: &DatabaseTransaction,
    user_id: i64,
) -> Result<Vec<conversation::Model>, String> {
    match conversation::Entity::find()
        .filter(conversation::Column::UserId.eq(user_id))
        .order_by(conversation::Column::UpdatedAt, sea_orm::Order::Desc)
        .all(tx)
        .await
    {
        Ok(model) => Ok(model),
        Err(e) => Err(format!("Error finding conversation by user_id: {}", e)),
    }
}

pub async fn find_by_user_id_and_conversation_id(
    tx: &DatabaseTransaction,
    user_id: i64,
    conversation_id: Uuid,
) -> Result<Option<conversation::Model>, String> {
    match conversation::Entity::find()
        .filter(conversation::Column::UserId.eq(user_id))
        .filter(conversation::Column::Id.eq(conversation_id))
        .one(tx)
        .await
    {
        Ok(model) => Ok(model),
        Err(e) => Err(format!(
            "Error finding conversation by user_id and conversation_id: {}",
            e
        )),
    }
}

pub async fn add_message(
    tx: &DatabaseTransaction,
    user_id: i64,
    conversation_id: Uuid,
    user_message_type: MessageType,
    user_message: String,
    transcription: Option<String>,
    images: Vec<String>,
    answer: String,
    message_id: i64,
) -> Result<conversation::Model, String> {
    let conversation_model = match conversation::Entity::find()
        .filter(conversation::Column::UserId.eq(user_id))
        .filter(conversation::Column::Id.eq(conversation_id))
        .one(tx)
        .await
    {
        Ok(Some(model)) => Ok(model),
        Ok(None) => Err(format!(
            "Not found the conversation by user_id and conversation_id"
        )),
        Err(e) => Err(format!("Error finding user by user_id: {}", e)),
    }?;

    let mut updated_conversation = conversation_model.conversation.clone();
    let mut conversation_title = conversation_model.title;
    if message_id < updated_conversation.len() as i64 {
        let _ = updated_conversation.split_off(message_id as usize);
    }
    if message_id == 0 {
        let words: Vec<&str> = user_message.split_whitespace().collect();
        let first_three_words = words
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<&str>>()
            .join(" ");

        if first_three_words.len() > 30 {
            conversation_title = conversation_title.chars().take(30).collect();
        } else {
            conversation_title = first_three_words;
        };
    }
    updated_conversation.push(
        serde_json::to_value(&Message {
            msgtype: user_message_type,
            id: updated_conversation.len() + 1,
            role: Role::User,
            content: user_message,
            transcription: transcription,
            images: images,
        })
        .map_err(|e| format!("Error to converting JSON Value from Message: {}", e))?,
    );
    updated_conversation.push(
        serde_json::to_value(&Message {
            msgtype: MessageType::Text,
            id: updated_conversation.len(),
            role: Role::Assistant,
            transcription: None,
            content: answer,
            images: vec![],
        })
        .map_err(|e| format!("Error to converting JSON Value from Message: {}", e))?,
    );

    let updated_model = conversation::ActiveModel {
        id: Set(conversation_model.id),
        user_id: Set(conversation_model.user_id),
        conversation: Set(updated_conversation),
        title: Set(conversation_title.clone()),
        created_at: Set(conversation_model.created_at),
        updated_at: Set(Utc::now()),
    };

    match updated_model.update(tx).await {
        Ok(model) => Ok(model),
        Err(e) => Err(format!("Error updating the conversation data: {}", e)),
    }
}

pub async fn edit_title(
    tx: &DatabaseTransaction,
    user_id: i64,
    conversation_id: Uuid,
    title: String,
) -> Result<conversation::Model, String> {
    let conversation_model = match conversation::Entity::find()
        .filter(conversation::Column::UserId.eq(user_id))
        .filter(conversation::Column::Id.eq(conversation_id))
        .one(tx)
        .await
    {
        Ok(Some(model)) => Ok(model),
        Ok(None) => Err(format!(
            "Not found the conversation by user_id and conversation_id"
        )),
        Err(e) => Err(format!("Error finding user by user_id: {}", e)),
    }?;

    let updated_model = conversation::ActiveModel {
        id: Set(conversation_model.id),
        user_id: Set(conversation_model.user_id),
        conversation: Set(conversation_model.conversation),
        title: Set(title),
        created_at: Set(conversation_model.created_at),
        updated_at: Set(Utc::now()),
    };

    match updated_model.update(tx).await {
        Ok(model) => Ok(model),
        Err(e) => Err(format!("Error updating the conversation title: {}", e)),
    }
}
