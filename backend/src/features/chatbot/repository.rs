#[async_trait::async_trait]
pub trait ChatRepository<C>: Send + Sync {
    async fn find_for_update(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::Chat>, anyhow::Error>;
    async fn save(
        &self,
        c: &mut C,
        chat: &super::domain::Chat,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct ChatRepositoryImpl {}

impl Default for ChatRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl ChatRepository<sqlx::MySqlConnection> for ChatRepositoryImpl {
    async fn find_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::Chat>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT chat_id, name, user_id, project_id,
                   messages AS "messages: sqlx::types::Json<Vec<super::domain::ChatMessage>>",
                   started_at, last_activity_at
            FROM chat
            WHERE chat_id = ?
            LIMIT 1
            FOR UPDATE
            "#,
            id.as_bytes().as_slice(),
        )
        .fetch_optional(c)
        .await?;

        match row {
            Some(r) => Ok(Some(super::domain::Chat {
                id: uuid::Uuid::from_slice(&r.chat_id)?,
                name: super::domain::ChatName(r.name),
                user_id: uuid::Uuid::from_slice(&r.user_id)?,
                project_id: uuid::Uuid::from_slice(&r.project_id)?,
                messages: r.messages.0,
                started_at: r.started_at.and_utc(),
                last_activity_at: r.last_activity_at.and_utc(),
            })),
            None => Ok(None),
        }
    }

    async fn save(
        &self,
        c: &mut sqlx::MySqlConnection,
        chat: &super::domain::Chat,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        sqlx::query!(
            r#"
            INSERT INTO chat
                (chat_id, name, user_id, project_id, messages, started_at, last_activity_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                name             = VALUES(name),
                messages         = VALUES(messages),
                last_activity_at = VALUES(last_activity_at)
            "#,
            chat.id.as_bytes().as_slice(),
            chat.name.0,
            chat.user_id.as_bytes().as_slice(),
            chat.project_id.as_bytes().as_slice(),
            sqlx::types::Json(&chat.messages) as _,
            chat.started_at,
            chat.last_activity_at,
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }

    async fn remove(
        &self,
        c: &mut sqlx::MySqlConnection,
        id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        sqlx::query!(
            "DELETE FROM chat WHERE chat_id = ?",
            id.as_bytes().as_slice(),
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }
}
