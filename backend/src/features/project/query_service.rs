pub struct ListProjectsByUserIdView {
    pub id: uuid::Uuid,
    pub created_by: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub indexed_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_file_id: uuid::Uuid,
}

pub enum ListProjectsByUserIdOrder {
    LastSeenAtAsc,
    LastSeenAtDesc,
}

pub struct ListProjectsByUserIdCursor {
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
    pub project_id: uuid::Uuid,
}

#[async_trait::async_trait]
pub trait QueryService: Send + Sync {
    async fn list_projects_by_user_id(
        &self,
        user_id: uuid::Uuid,
        order: ListProjectsByUserIdOrder,
        cursor: Option<ListProjectsByUserIdCursor>,
        limit: u32,
    ) -> Result<Vec<ListProjectsByUserIdView>, anyhow::Error>;
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
    async fn list_projects_by_user_id(
        &self,
        user_id: uuid::Uuid,
        order: ListProjectsByUserIdOrder,
        cursor: Option<ListProjectsByUserIdCursor>,
        limit: u32,
    ) -> Result<Vec<ListProjectsByUserIdView>, anyhow::Error> {
        let limit = limit as i64;

        let rows = match (order, cursor) {
            (ListProjectsByUserIdOrder::LastSeenAtAsc, Some(c)) => sqlx::query!(
                r#"
                    SELECT project_id, created_by, name, description,
                           indexed_at, last_seen_at, last_seen_file_id
                    FROM project
                    WHERE created_by = ?
                      AND (last_seen_at, project_id) > (?, ?)
                    ORDER BY last_seen_at ASC, project_id ASC
                    LIMIT ?
                    "#,
                user_id.as_bytes().as_slice(),
                c.last_seen_at,
                c.project_id.as_bytes().as_slice(),
                limit,
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(ListProjectsByUserIdView {
                    id: uuid::Uuid::from_slice(&r.project_id)?,
                    created_by: uuid::Uuid::from_slice(&r.created_by)?,
                    name: r.name,
                    description: r.description,
                    indexed_at: r.indexed_at.and_utc(),
                    last_seen_at: r.last_seen_at.and_utc(),
                    last_seen_file_id: uuid::Uuid::from_slice(&r.last_seen_file_id)?,
                })
            })
            .collect::<Result<Vec<_>, anyhow::Error>>()?,
            (ListProjectsByUserIdOrder::LastSeenAtAsc, None) => sqlx::query!(
                r#"
                    SELECT project_id, created_by, name, description,
                           indexed_at, last_seen_at, last_seen_file_id
                    FROM project
                    WHERE created_by = ?
                    ORDER BY last_seen_at ASC, project_id ASC
                    LIMIT ?
                    "#,
                user_id.as_bytes().as_slice(),
                limit,
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(ListProjectsByUserIdView {
                    id: uuid::Uuid::from_slice(&r.project_id)?,
                    created_by: uuid::Uuid::from_slice(&r.created_by)?,
                    name: r.name,
                    description: r.description,
                    indexed_at: r.indexed_at.and_utc(),
                    last_seen_at: r.last_seen_at.and_utc(),
                    last_seen_file_id: uuid::Uuid::from_slice(&r.last_seen_file_id)?,
                })
            })
            .collect::<Result<Vec<_>, anyhow::Error>>()?,
            (ListProjectsByUserIdOrder::LastSeenAtDesc, Some(c)) => sqlx::query!(
                r#"
                    SELECT project_id, created_by, name, description,
                           indexed_at, last_seen_at, last_seen_file_id
                    FROM project
                    WHERE created_by = ?
                      AND (last_seen_at, project_id) < (?, ?)
                    ORDER BY last_seen_at DESC, project_id DESC
                    LIMIT ?
                    "#,
                user_id.as_bytes().as_slice(),
                c.last_seen_at,
                c.project_id.as_bytes().as_slice(),
                limit,
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(ListProjectsByUserIdView {
                    id: uuid::Uuid::from_slice(&r.project_id)?,
                    created_by: uuid::Uuid::from_slice(&r.created_by)?,
                    name: r.name,
                    description: r.description,
                    indexed_at: r.indexed_at.and_utc(),
                    last_seen_at: r.last_seen_at.and_utc(),
                    last_seen_file_id: uuid::Uuid::from_slice(&r.last_seen_file_id)?,
                })
            })
            .collect::<Result<Vec<_>, anyhow::Error>>()?,
            (ListProjectsByUserIdOrder::LastSeenAtDesc, None) => sqlx::query!(
                r#"
                    SELECT project_id, created_by, name, description,
                           indexed_at, last_seen_at, last_seen_file_id
                    FROM project
                    WHERE created_by = ?
                    ORDER BY last_seen_at DESC, project_id DESC
                    LIMIT ?
                    "#,
                user_id.as_bytes().as_slice(),
                limit,
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(ListProjectsByUserIdView {
                    id: uuid::Uuid::from_slice(&r.project_id)?,
                    created_by: uuid::Uuid::from_slice(&r.created_by)?,
                    name: r.name,
                    description: r.description,
                    indexed_at: r.indexed_at.and_utc(),
                    last_seen_at: r.last_seen_at.and_utc(),
                    last_seen_file_id: uuid::Uuid::from_slice(&r.last_seen_file_id)?,
                })
            })
            .collect::<Result<Vec<_>, anyhow::Error>>()?,
        };

        Ok(rows)
    }
}
