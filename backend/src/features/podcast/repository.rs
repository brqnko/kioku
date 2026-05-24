#[async_trait::async_trait]
pub trait PodcastRepository<C>: Send + Sync {
    async fn find_for_update(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::Podcast>, anyhow::Error>;
    async fn save(
        &self,
        c: &mut C,
        podcast: &super::domain::Podcast,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct PodcastRepositoryImpl {}

impl Default for PodcastRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl PodcastRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl PodcastRepository<sqlx::MySqlConnection> for PodcastRepositoryImpl {
    async fn find_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::Podcast>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT podcast_id, name, description, user_id, project_id,
                   used_file_ids AS "used_file_ids: sqlx::types::Json<Vec<uuid::Uuid>>",
                   podcast_script AS "podcast_script: sqlx::types::Json<Vec<super::domain::PodcastScriptEntry>>",
                   audio_storage_id,
                   podcast_created_at
            FROM podcast
            WHERE podcast_id = ?
            LIMIT 1
            FOR UPDATE
            "#,
            id.as_bytes().as_slice(),
        )
        .fetch_optional(c)
        .await?;

        match row {
            Some(r) => Ok(Some(super::domain::Podcast {
                id: uuid::Uuid::from_slice(&r.podcast_id)?,
                name: super::domain::PodcastName(r.name),
                description: super::domain::PodcastDescription(r.description),
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

    async fn save(
        &self,
        c: &mut sqlx::MySqlConnection,
        podcast: &super::domain::Podcast,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        sqlx::query!(
            r#"
            INSERT INTO podcast
                (podcast_id, name, description, user_id, project_id,
                 used_file_ids, podcast_script, audio_storage_id, podcast_created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                name               = VALUES(name),
                description        = VALUES(description),
                used_file_ids      = VALUES(used_file_ids),
                podcast_script     = VALUES(podcast_script),
                audio_storage_id   = VALUES(audio_storage_id)
            "#,
            podcast.id.as_bytes().as_slice(),
            podcast.name.0,
            podcast.description.0,
            podcast.user_id.as_bytes().as_slice(),
            podcast.project_id.as_bytes().as_slice(),
            sqlx::types::Json(&podcast.used_file_ids) as _,
            sqlx::types::Json(&podcast.podcast_script) as _,
            podcast.audio_storage_id.as_bytes().as_slice(),
            podcast.podcast_created_at,
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
            "DELETE FROM podcast WHERE podcast_id = ?",
            id.as_bytes().as_slice(),
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }
}
