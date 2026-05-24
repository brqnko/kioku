#[async_trait::async_trait]
pub trait ProjectRepository<C>: Send + Sync {
    async fn find_for_update(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::Project>, anyhow::Error>;
    async fn save(
        &self,
        c: &mut C,
        project: &super::domain::Project,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct ProjectRepositoryImpl {}

impl Default for ProjectRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl ProjectRepository<sqlx::MySqlConnection> for ProjectRepositoryImpl {
    async fn find_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::Project>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT project_id, created_by, name, description,
                   indexed_at, last_seen_at, last_seen_file_id
            FROM project
            WHERE project_id = ?
            LIMIT 1
            FOR UPDATE
            "#,
            id.as_bytes().as_slice(),
        )
        .fetch_optional(c)
        .await?;

        match row {
            Some(r) => Ok(Some(super::domain::Project {
                id: uuid::Uuid::from_slice(&r.project_id)?,
                created_by: uuid::Uuid::from_slice(&r.created_by)?,
                name: super::domain::ProjectName(r.name),
                description: super::domain::ProjectDescription(r.description),
                indexed_at: r.indexed_at.and_utc(),
                last_seen_at: r.last_seen_at.and_utc(),
                last_seen_file_id: uuid::Uuid::from_slice(&r.last_seen_file_id)?,
            })),
            None => Ok(None),
        }
    }

    async fn save(
        &self,
        c: &mut sqlx::MySqlConnection,
        project: &super::domain::Project,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        sqlx::query!(
            r#"
            INSERT INTO project
                (project_id, created_by, name, description,
                 indexed_at, last_seen_at, last_seen_file_id)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                name              = VALUES(name),
                description       = VALUES(description),
                indexed_at        = VALUES(indexed_at),
                last_seen_at      = VALUES(last_seen_at),
                last_seen_file_id = VALUES(last_seen_file_id)
            "#,
            project.id.as_bytes().as_slice(),
            project.created_by.as_bytes().as_slice(),
            project.name.0,
            project.description.0,
            project.indexed_at,
            project.last_seen_at,
            project.last_seen_file_id.as_bytes().as_slice(),
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
            "DELETE FROM project WHERE project_id = ?",
            id.as_bytes().as_slice(),
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }
}
