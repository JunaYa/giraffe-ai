use crate::{AppError, AppState};

use super::{Chat, ChatType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
}

#[allow(unused)]
impl AppState {
    pub async fn create_chat(&self, input: CreateChat, ws_id: i64) -> Result<Chat, AppError> {
        let len = input.members.len();
        if len < 2 {
            return Err(AppError::CreateChatError(
                "At least 2 members are required".to_string(),
            ));
        }

        if len > 8 && input.name.is_none() {
            return Err(AppError::CreateChatError(
                "Name is required for group chats with more than 8 members".to_string(),
            ));
        }

        // verify if all members exist
        let users = self.fetch_by_ids(&input.members).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                "Some members do not exist".to_string(),
            ));
        }

        let chat_type = match (&input.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };

        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(ws_id)
        .bind(input.name)
        .bind(chat_type)
        .bind(input.members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn fetch_all_chats(&self, ws_id: i64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(chats)
    }

    pub async fn fetch_chat_by_id(&self, id: i64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn update_chat(&self, id: i64, input: UpdateChat) -> Result<Chat, AppError> {
        let len = input.members.len();
        if len < 2 {
            return Err(AppError::UpdateChatError(
                "At least 2 members are required".to_string(),
            ));
        }

        if len > 8 && input.name.is_none() {
            return Err(AppError::CreateChatError(
                "Name is required for group chats with more than 8 members".to_string(),
            ));
        }
        let chat = sqlx::query_as(
            r#"
            UPDATE chats
            SET name = $2, members = $3
            WHERE id = $1
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(id)
        .bind(input.name)
        .bind(input.members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn delete_chat(&self, id: i64, ws_id: i64) -> Result<(), AppError> {
        let chat = self.fetch_chat_by_id(id).await?;
        if chat.is_none() {
            return Err(AppError::DeleteChatError("Chat not found".to_string()));
        }

        if chat.unwrap().ws_id != ws_id {
            return Err(AppError::DeleteChatError(
                "You are not allowed to delete this chat".to_string(),
            ));
        }

        sqlx::query(
            r#"
            DELETE FROM chats WHERE id = $1 AND ws_id = $2
            "#,
        )
        .bind(id)
        .bind(ws_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
impl CreateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };

        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[tokio::test]
    async fn create_single_chat_should_work() -> Result<()> {
        let (_tdc, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = state
            .create_chat(input, 1)
            .await
            .expect("create chat failed");
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::Single);
        Ok(())
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() -> Result<()> {
        let (_tdc, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("general", &[1, 2, 3], true);
        let chat = state
            .create_chat(input, 1)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(())
    }

    #[tokio::test]
    async fn chat_get_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat = state
            .fetch_chat_by_id(1)
            .await
            .expect("fetch chat failed")
            .unwrap();
        assert_eq!(chat.id, 1);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members.len(), 5);
        assert_eq!(chat.name, Some("general".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn chat_fetch_all_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chats = state.fetch_all_chats(1).await.expect("fetch chats failed");
        assert_eq!(chats.len(), 4);
        Ok(())
    }

    #[tokio::test]
    async fn chat_update_name_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = UpdateChat {
            name: Some("general1".to_string()),
            members: vec![1, 2, 3],
        };
        let chat = state
            .update_chat(1, input)
            .await
            .expect("update chat failed");
        assert_eq!(chat.name, Some("general1".to_string()));
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(())
    }

    #[tokio::test]
    async fn chat_update_members_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = UpdateChat {
            name: None,
            members: vec![1, 2, 3],
        };
        let chat = state
            .update_chat(1, input)
            .await
            .expect("update chat failed");
        assert_eq!(chat.name, None);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(())
    }

    #[tokio::test]
    async fn chat_delete_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        state.delete_chat(1, 1).await.expect("delete chat failed");

        let chat = state.fetch_chat_by_id(1).await.expect("fetch chat failed");
        assert!(chat.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn chat_delete_should_fail_if_not_found() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let result = state.delete_chat(1, 2).await;
        assert!(result.is_err());
        Ok(())
    }
}
