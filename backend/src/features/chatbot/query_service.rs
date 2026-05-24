pub struct GetChatView {
    pub id: uuid::Uuid,
    pub name: String,
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub messages: Vec<super::domain::ChatMessage>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
}

pub struct ListChatsByProjectView {
    pub id: uuid::Uuid,
    pub name: String,
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
}

pub struct ListChatsByProjectCursor {
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub chat_id: uuid::Uuid,
}

#[async_trait::async_trait]
pub trait QueryService: Send + Sync {
    async fn get_chat(&self, chat_id: uuid::Uuid) -> Result<Option<GetChatView>, anyhow::Error>;
    async fn list_chats_by_project(
        &self,
        project_id: uuid::Uuid,
        cursor: Option<ListChatsByProjectCursor>,
        limit: u32,
    ) -> Result<Vec<ListChatsByProjectView>, anyhow::Error>;
    async fn count_by_project(&self, project_id: uuid::Uuid) -> Result<u64, anyhow::Error>;
}

pub struct QueryServiceImpl {
    pool: sqlx::Pool<sqlx::MySql>,
}

impl QueryServiceImpl {
    pub fn new(pool: sqlx::Pool<sqlx::MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl QueryService for QueryServiceImpl {
    async fn get_chat(&self, chat_id: uuid::Uuid) -> Result<Option<GetChatView>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT chat_id, name, user_id, project_id,
                   messages AS "messages: sqlx::types::Json<Vec<super::domain::ChatMessage>>",
                   started_at, last_activity_at
            FROM chat
            WHERE chat_id = ?
            LIMIT 1
            "#,
            chat_id.as_bytes().as_slice(),
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(GetChatView {
                id: uuid::Uuid::from_slice(&r.chat_id)?,
                name: r.name,
                user_id: uuid::Uuid::from_slice(&r.user_id)?,
                project_id: uuid::Uuid::from_slice(&r.project_id)?,
                messages: r.messages.0,
                started_at: r.started_at.and_utc(),
                last_activity_at: r.last_activity_at.and_utc(),
            })),
            None => Ok(None),
        }
    }

    async fn list_chats_by_project(
        &self,
        project_id: uuid::Uuid,
        cursor: Option<ListChatsByProjectCursor>,
        limit: u32,
    ) -> Result<Vec<ListChatsByProjectView>, anyhow::Error> {
        let limit = limit as i64;
        let rows = match cursor {
            Some(c) => sqlx::query!(
                r#"
                SELECT chat_id, name, user_id, project_id,
                       started_at, last_activity_at
                FROM chat
                WHERE project_id = ?
                  AND (last_activity_at, chat_id) < (?, ?)
                ORDER BY last_activity_at DESC, chat_id DESC
                LIMIT ?
                "#,
                project_id.as_bytes().as_slice(),
                c.last_activity_at,
                c.chat_id.as_bytes().as_slice(),
                limit,
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(ListChatsByProjectView {
                    id: uuid::Uuid::from_slice(&r.chat_id)?,
                    name: r.name,
                    user_id: uuid::Uuid::from_slice(&r.user_id)?,
                    project_id: uuid::Uuid::from_slice(&r.project_id)?,
                    started_at: r.started_at.and_utc(),
                    last_activity_at: r.last_activity_at.and_utc(),
                })
            })
            .collect::<Result<Vec<ListChatsByProjectView>, anyhow::Error>>()?,
            None => sqlx::query!(
                r#"
                SELECT chat_id, name, user_id, project_id,
                       started_at, last_activity_at
                FROM chat
                WHERE project_id = ?
                ORDER BY last_activity_at DESC, chat_id DESC
                LIMIT ?
                "#,
                project_id.as_bytes().as_slice(),
                limit,
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(ListChatsByProjectView {
                    id: uuid::Uuid::from_slice(&r.chat_id)?,
                    name: r.name,
                    user_id: uuid::Uuid::from_slice(&r.user_id)?,
                    project_id: uuid::Uuid::from_slice(&r.project_id)?,
                    started_at: r.started_at.and_utc(),
                    last_activity_at: r.last_activity_at.and_utc(),
                })
            })
            .collect::<Result<Vec<ListChatsByProjectView>, anyhow::Error>>()?,
        };
        Ok(rows)
    }

    async fn count_by_project(&self, project_id: uuid::Uuid) -> Result<u64, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS count
            FROM chat
            WHERE project_id = ?
            "#,
            project_id.as_bytes().as_slice(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count as u64)
    }
}
