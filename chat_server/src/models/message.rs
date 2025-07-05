use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{AppError, AppState, ChatFile, Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

impl AppState {
    #[allow(dead_code)]
    pub async fn create_message(
        &self,
        input: CreateMessage,
        chat_id: i64,
        user_id: i64,
    ) -> Result<Message, AppError> {
        let base_dir = &self.config.server.base_dir;
        // verify content is not empty
        if input.content.is_empty() {
            return Err(AppError::CreateMessageError("Content is empty".to_string()));
        }

        // verify files is exists
        for s in &input.files {
            let file = ChatFile::from_str(s)?;
            if !file.path(base_dir).exists() {
                return Err(AppError::CreateMessageError(format!(
                    "File {} does not exist",
                    s
                )));
            }
        }

        // create message
        let message: Message = sqlx::query_as(
            r#"
        INSERT INTO messages (chat_id, sender_id, content, files)
        VALUES ($1, $2, $3, $4)
        RETURNING id, chat_id, sender_id, content, files, created_at
        "#,
        )
        .bind(chat_id)
        .bind(user_id)
        .bind(input.content)
        .bind(&input.files)
        .fetch_one(&self.pool)
        .await?;

        Ok(message)
    }
}
