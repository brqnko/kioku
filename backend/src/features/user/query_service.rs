pub struct GetUserProfileView {
    pub id: uuid::Uuid,
    pub display_name: String,
    pub language_code: String,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

pub struct ListRefreshTokensByUserIdView {
    pub id: uuid::Uuid,
    pub ip_address: String,
    pub user_agent: String,
    pub activated_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct GetDashboardRecentFileView {
    pub id: uuid::Uuid,
    pub name: String,
    pub user_id: uuid::Uuid,
    pub storage_type: u8,
    pub storage_id: uuid::Uuid,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

pub struct GetDashboardView {
    pub ai_learning_summary: String,
    pub ai_learning_summary_updated_at: chrono::DateTime<chrono::Utc>,
    pub recent_seen_files: Vec<GetDashboardRecentFileView>,
}

pub struct GetRateLimitsView {
    pub podcast_daily_count: u32,
    pub podcast_daily_count_reset_at: chrono::DateTime<chrono::Utc>,
    pub chatbot_daily_count: u32,
    pub chatbot_daily_count_reset_at: chrono::DateTime<chrono::Utc>,
    pub file_upload_daily_count: u32,
    pub file_upload_daily_count_reset_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait::async_trait]
pub trait QueryService: Send + Sync {
    async fn get_user_profile(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Option<GetUserProfileView>, anyhow::Error>;
    async fn list_refresh_tokens_by_user_id(
        &self,
        user_id: uuid::Uuid,
        cursor: Option<uuid::Uuid>,
        limit: u32,
    ) -> Result<Vec<ListRefreshTokensByUserIdView>, anyhow::Error>;
    async fn get_dashboard(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Option<GetDashboardView>, anyhow::Error>;
    async fn get_rate_limits(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Option<GetRateLimitsView>, anyhow::Error>;
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
    async fn get_user_profile(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Option<GetUserProfileView>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT user_id, display_name, language_code, joined_at
            FROM user
            WHERE user_id = ?
            LIMIT 1
            "#,
            user_id.as_bytes().as_slice(),
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(GetUserProfileView {
                id: uuid::Uuid::from_slice(&r.user_id)?,
                display_name: r.display_name,
                language_code: r.language_code,
                joined_at: r.joined_at.and_utc(),
            })),
            None => Ok(None),
        }
    }

    async fn list_refresh_tokens_by_user_id(
        &self,
        user_id: uuid::Uuid,
        cursor: Option<uuid::Uuid>,
        limit: u32,
    ) -> Result<Vec<ListRefreshTokensByUserIdView>, anyhow::Error> {
        let limit = limit as i64;
        let rows = match cursor {
            Some(cursor) => sqlx::query!(
                r#"
                    SELECT refresh_token_id, ip_address, user_agent,
                           activated_at, last_used_at, expires_at
                    FROM refresh_token
                    WHERE user_id = ? AND refresh_token_id > ?
                    ORDER BY refresh_token_id ASC
                    LIMIT ?
                    "#,
                user_id.as_bytes().as_slice(),
                cursor.as_bytes().as_slice(),
                limit,
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(ListRefreshTokensByUserIdView {
                    id: uuid::Uuid::from_slice(&r.refresh_token_id)?,
                    ip_address: r.ip_address,
                    user_agent: r.user_agent,
                    activated_at: r.activated_at.and_utc(),
                    last_used_at: r.last_used_at.and_utc(),
                    expires_at: r.expires_at.and_utc(),
                })
            })
            .collect::<Result<Vec<_>, anyhow::Error>>()?,
            None => sqlx::query!(
                r#"
                    SELECT refresh_token_id, ip_address, user_agent,
                           activated_at, last_used_at, expires_at
                    FROM refresh_token
                    WHERE user_id = ?
                    ORDER BY refresh_token_id ASC
                    LIMIT ?
                    "#,
                user_id.as_bytes().as_slice(),
                limit,
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(ListRefreshTokensByUserIdView {
                    id: uuid::Uuid::from_slice(&r.refresh_token_id)?,
                    ip_address: r.ip_address,
                    user_agent: r.user_agent,
                    activated_at: r.activated_at.and_utc(),
                    last_used_at: r.last_used_at.and_utc(),
                    expires_at: r.expires_at.and_utc(),
                })
            })
            .collect::<Result<Vec<_>, anyhow::Error>>()?,
        };

        Ok(rows)
    }

    async fn get_dashboard(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Option<GetDashboardView>, anyhow::Error> {
        let user_row = sqlx::query!(
            r#"
            SELECT
                ai_learning_summary,
                ai_learning_summary_updated_at,
                recent_seen_file_ids as "recent_seen_file_ids: sqlx::types::Json<Vec<uuid::Uuid>>"
            FROM user
            WHERE user_id = ?
            LIMIT 1
            "#,
            user_id.as_bytes().as_slice(),
        )
        .fetch_optional(&self.pool)
        .await?;

        let user_row = match user_row {
            Some(r) => r,
            None => return Ok(None),
        };

        let recent_ids = user_row.recent_seen_file_ids.0;

        let recent_seen_files = if recent_ids.is_empty() {
            Vec::new()
        } else {
            let mut builder = sqlx::QueryBuilder::<sqlx::MySql>::new(
                "SELECT file_id, name, user_id, storage_type, storage_id, changed_at \
                 FROM file WHERE file_id IN (",
            );
            let mut separated = builder.separated(", ");
            for id in &recent_ids {
                separated.push_bind(id.as_bytes().to_vec());
            }
            separated.push_unseparated(")");

            let rows = builder.build().fetch_all(&self.pool).await?;

            use sqlx::Row as _;
            let mut by_id =
                std::collections::HashMap::<uuid::Uuid, GetDashboardRecentFileView>::new();
            for r in rows {
                let file_id: Vec<u8> = r.try_get("file_id")?;
                let owner_id: Vec<u8> = r.try_get("user_id")?;
                let storage_id: Vec<u8> = r.try_get("storage_id")?;
                let changed_at: chrono::NaiveDateTime = r.try_get("changed_at")?;
                let id = uuid::Uuid::from_slice(&file_id)?;
                by_id.insert(
                    id,
                    GetDashboardRecentFileView {
                        id,
                        name: r.try_get("name")?,
                        user_id: uuid::Uuid::from_slice(&owner_id)?,
                        storage_type: r.try_get("storage_type")?,
                        storage_id: uuid::Uuid::from_slice(&storage_id)?,
                        changed_at: changed_at.and_utc(),
                    },
                );
            }

            recent_ids
                .into_iter()
                .filter_map(|id| by_id.remove(&id))
                .collect::<Vec<GetDashboardRecentFileView>>()
        };

        Ok(Some(GetDashboardView {
            ai_learning_summary: user_row.ai_learning_summary,
            ai_learning_summary_updated_at: user_row.ai_learning_summary_updated_at.and_utc(),
            recent_seen_files,
        }))
    }

    async fn get_rate_limits(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Option<GetRateLimitsView>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT
                podcast_daily_count,
                podcast_daily_count_reset_at,
                chatbot_daily_count,
                chatbot_daily_count_reset_at,
                file_upload_daily_count,
                file_upload_daily_count_reset_at
            FROM user
            WHERE user_id = ?
            LIMIT 1
            "#,
            user_id.as_bytes().as_slice(),
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(GetRateLimitsView {
                podcast_daily_count: r.podcast_daily_count,
                podcast_daily_count_reset_at: r.podcast_daily_count_reset_at.and_utc(),
                chatbot_daily_count: r.chatbot_daily_count,
                chatbot_daily_count_reset_at: r.chatbot_daily_count_reset_at.and_utc(),
                file_upload_daily_count: r.file_upload_daily_count,
                file_upload_daily_count_reset_at: r.file_upload_daily_count_reset_at.and_utc(),
            })),
            None => Ok(None),
        }
    }
}
