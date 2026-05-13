pub struct GetPodcastView {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub used_file_ids: Vec<uuid::Uuid>,
    pub podcast_script: Vec<super::domain::PodcastScriptEntry>,
    pub audio_storage_id: uuid::Uuid,
    pub podcast_created_at: chrono::DateTime<chrono::Utc>,
}

pub struct ListPodcastsByProjectView {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub podcast_created_at: chrono::DateTime<chrono::Utc>,
}

pub struct ListPodcastsByProjectCursor {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub podcast_id: uuid::Uuid,
}

#[async_trait::async_trait]
pub trait QueryService: Send + Sync {
    async fn get_podcast(
        &self,
        podcast_id: uuid::Uuid,
    ) -> Result<Option<GetPodcastView>, anyhow::Error>;
    async fn list_podcasts_by_project(
        &self,
        project_id: uuid::Uuid,
        cursor: Option<ListPodcastsByProjectCursor>,
        limit: u32,
    ) -> Result<Vec<ListPodcastsByProjectView>, anyhow::Error>;
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
    async fn get_podcast(
        &self,
        podcast_id: uuid::Uuid,
    ) -> Result<Option<GetPodcastView>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT podcast_id, name, description, user_id, project_id,
                   used_file_ids AS "used_file_ids: sqlx::types::Json<Vec<uuid::Uuid>>",
                   podcast_script AS "podcast_script: sqlx::types::Json<Vec<super::domain::PodcastScriptEntry>>",
                   audio_storage_id,
                   podcast_created_at
            FROM podcast
            WHERE podcast_id = ?
            "#,
            podcast_id.as_bytes().as_slice(),
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(GetPodcastView {
                id: uuid::Uuid::from_slice(&r.podcast_id)?,
                name: r.name,
                description: r.description,
                user_id: uuid::Uuid::from_slice(&r.user_id)?,
                project_id: uuid::Uuid::from_slice(&r.project_id)?,
                used_file_ids: r.used_file_ids.0,
                podcast_script: r.podcast_script.0,
                audio_storage_id: uuid::Uuid::from_slice(&r.audio_storage_id)?,
                podcast_created_at: r.podcast_created_at.and_utc(),
            })),
            None => Ok(None),
        }
    }

    async fn list_podcasts_by_project(
        &self,
        project_id: uuid::Uuid,
        cursor: Option<ListPodcastsByProjectCursor>,
        limit: u32,
    ) -> Result<Vec<ListPodcastsByProjectView>, anyhow::Error> {
        let limit = limit as i64;
        let rows = match cursor {
            Some(c) => sqlx::query!(
                r#"
                SELECT podcast_id, name, description, user_id, project_id,
                       podcast_created_at
                FROM podcast
                WHERE project_id = ?
                  AND (podcast_created_at, podcast_id) < (?, ?)
                ORDER BY podcast_created_at DESC, podcast_id DESC
                LIMIT ?
                "#,
                project_id.as_bytes().as_slice(),
                c.created_at,
                c.podcast_id.as_bytes().as_slice(),
                limit,
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(ListPodcastsByProjectView {
                    id: uuid::Uuid::from_slice(&r.podcast_id)?,
                    name: r.name,
                    description: r.description,
                    user_id: uuid::Uuid::from_slice(&r.user_id)?,
                    project_id: uuid::Uuid::from_slice(&r.project_id)?,
                    podcast_created_at: r.podcast_created_at.and_utc(),
                })
            })
            .collect::<Result<Vec<ListPodcastsByProjectView>, anyhow::Error>>()?,
            None => sqlx::query!(
                r#"
                SELECT podcast_id, name, description, user_id, project_id,
                       podcast_created_at
                FROM podcast
                WHERE project_id = ?
                ORDER BY podcast_created_at DESC, podcast_id DESC
                LIMIT ?
                "#,
                project_id.as_bytes().as_slice(),
                limit,
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(ListPodcastsByProjectView {
                    id: uuid::Uuid::from_slice(&r.podcast_id)?,
                    name: r.name,
                    description: r.description,
                    user_id: uuid::Uuid::from_slice(&r.user_id)?,
                    project_id: uuid::Uuid::from_slice(&r.project_id)?,
                    podcast_created_at: r.podcast_created_at.and_utc(),
                })
            })
            .collect::<Result<Vec<ListPodcastsByProjectView>, anyhow::Error>>()?,
        };
        Ok(rows)
    }

    async fn count_by_project(&self, project_id: uuid::Uuid) -> Result<u64, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS count
            FROM podcast
            WHERE project_id = ?
            "#,
            project_id.as_bytes().as_slice(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count as u64)
    }
}
