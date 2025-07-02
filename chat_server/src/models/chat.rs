use crate::AppError;

use super::{Chat, ChatType, ChatUser};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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
impl Chat {
    pub async fn create(input: CreateChat, ws_id: i64, pool: &PgPool) -> Result<Self, AppError> {
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
        let users = ChatUser::fetch_by_ids(&input.members, pool).await?;
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
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    pub async fn fetch_all(ws_id: i64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id)
        .fetch_all(pool)
        .await?;

        Ok(chats)
    }

    pub async fn fetch_by_id(id: i64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(chat)
    }

    pub async fn update(id: i64, input: UpdateChat, pool: &PgPool) -> Result<Self, AppError> {
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
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    pub async fn delete(id: i64, ws_id: i64, pool: &PgPool) -> Result<(), AppError> {
        let chat = Self::fetch_by_id(id, pool).await?;
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
        .execute(pool)
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
    use crate::test_utils::get_test_pool;

    use super::*;

    #[tokio::test]
    async fn create_single_chat_should_work() {
        let (_tdc, pool) = get_test_pool(None).await;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = Chat::create(input, 1, &pool)
            .await
            .expect("Failed to create chat");
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::Single);
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let input = CreateChat::new("general", &[1, 2, 3], true);
        let chat = Chat::create(input, 1, &pool)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
    }

    #[tokio::test]
    async fn chat_get_by_id_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let chat = Chat::fetch_by_id(1, &pool)
            .await
            .expect("fetch chat failed")
            .unwrap();
        assert_eq!(chat.id, 1);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members.len(), 5);
        assert_eq!(chat.name, Some("general".to_string()));
    }

    #[tokio::test]
    async fn chat_fetch_all_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let chats = Chat::fetch_all(1, &pool).await.expect("fetch chats failed");
        assert_eq!(chats.len(), 4);
    }

    #[tokio::test]
    async fn chat_update_name_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let input = UpdateChat {
            name: Some("general1".to_string()),
            members: vec![1, 2, 3],
        };
        let chat = Chat::update(1, input, &pool)
            .await
            .expect("update chat failed");
        assert_eq!(chat.name, Some("general1".to_string()));
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
    }

    #[tokio::test]
    async fn chat_update_members_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let input = UpdateChat {
            name: None,
            members: vec![1, 2, 3],
        };
        let chat = Chat::update(1, input, &pool)
            .await
            .expect("update chat failed");
        assert_eq!(chat.name, None);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
    }

    #[tokio::test]
    async fn chat_delete_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        Chat::delete(1, 1, &pool).await.expect("delete chat failed");

        let chat = Chat::fetch_by_id(1, &pool)
            .await
            .expect("fetch chat failed");
        assert!(chat.is_none());
    }

    #[tokio::test]
    async fn chat_delete_should_fail_if_not_found() {
        let (_tdb, pool) = get_test_pool(None).await;
        let result = Chat::delete(1, 2, &pool).await;
        assert!(result.is_err());
    }
}
