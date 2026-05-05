pub struct ListChildrenByParentFolderView {
    pub id: uuid::Uuid,
    pub parent_id: uuid::Uuid,
    pub parent_kind: u8,
    pub name: String,
    pub description: String,
    pub user_id: uuid::Uuid,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

pub struct ListChildrenByParentFileView {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub user_id: uuid::Uuid,
    pub storage_id: uuid::Uuid,
    pub file_size: u64,
    pub parent_id: uuid::Uuid,
    pub parent_kind: u8,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

pub enum ListChildrenByParentView {
    Folder(ListChildrenByParentFolderView),
    File(ListChildrenByParentFileView),
}

pub enum ListChildrenByParentPhase {
    Folders,
    Files,
}

pub struct ListChildrenByParentCursor {
    pub phase: ListChildrenByParentPhase,
    pub name: String,
    pub id: uuid::Uuid,
}

pub struct GetFileView {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub user_id: uuid::Uuid,
    pub storage_type: u8,
    pub storage_id: uuid::Uuid,
    pub file_size: u64,
    pub parent_id: uuid::Uuid,
    pub parent_kind: u8,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

pub struct GetFolderView {
    pub id: uuid::Uuid,
    pub parent_id: uuid::Uuid,
    pub parent_kind: u8,
    pub depth: u8,
    pub name: String,
    pub description: String,
    pub user_id: uuid::Uuid,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

pub struct GetTextContentView {
    pub content: String,
}

#[async_trait::async_trait]
pub trait QueryService: Send + Sync {
    async fn list_children_by_parent(
        &self,
        parent: &super::domain::ParentId,
        cursor: Option<ListChildrenByParentCursor>,
        limit: u32,
    ) -> Result<Vec<ListChildrenByParentView>, anyhow::Error>;

    async fn get_file(&self, file_id: uuid::Uuid) -> Result<Option<GetFileView>, anyhow::Error>;

    async fn get_folder(
        &self,
        folder_id: uuid::Uuid,
    ) -> Result<Option<GetFolderView>, anyhow::Error>;

    async fn get_text_content(
        &self,
        storage_id: uuid::Uuid,
    ) -> Result<Option<GetTextContentView>, anyhow::Error>;
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
    async fn list_children_by_parent(
        &self,
        parent: &super::domain::ParentId,
        cursor: Option<ListChildrenByParentCursor>,
        limit: u32,
    ) -> Result<Vec<ListChildrenByParentView>, anyhow::Error> {
        let parent_id = parent.id().as_bytes().to_vec();
        let parent_kind = parent.kind();
        let limit = limit as i64;

        let (start_with_folders, folder_cursor, file_cursor) = match cursor {
            None => (true, None, None),
            Some(ListChildrenByParentCursor {
                phase: ListChildrenByParentPhase::Folders,
                name,
                id,
            }) => (true, Some((name, id.as_bytes().to_vec())), None),
            Some(ListChildrenByParentCursor {
                phase: ListChildrenByParentPhase::Files,
                name,
                id,
            }) => (false, None, Some((name, id.as_bytes().to_vec()))),
        };

        let mut items: Vec<ListChildrenByParentView> = Vec::new();

        if start_with_folders {
            let folders: Vec<ListChildrenByParentFolderView> = match folder_cursor {
                Some((cursor_name, cursor_id)) => sqlx::query!(
                    r#"
                    SELECT folder_id, parent_id, parent_kind,
                           name, description, user_id,
                           uploaded_at, changed_at
                    FROM folder
                    WHERE parent_kind = ? AND parent_id = ?
                      AND (name, folder_id) > (?, ?)
                    ORDER BY name ASC, folder_id ASC
                    LIMIT ?
                    "#,
                    parent_kind,
                    parent_id,
                    cursor_name,
                    cursor_id,
                    limit,
                )
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|r| {
                    Ok(ListChildrenByParentFolderView {
                        id: uuid::Uuid::from_slice(&r.folder_id)?,
                        parent_id: uuid::Uuid::from_slice(&r.parent_id)?,
                        parent_kind: r.parent_kind,
                        name: r.name,
                        description: r.description,
                        user_id: uuid::Uuid::from_slice(&r.user_id)?,
                        uploaded_at: r.uploaded_at.and_utc(),
                        changed_at: r.changed_at.and_utc(),
                    })
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?,
                None => sqlx::query!(
                    r#"
                    SELECT folder_id, parent_id, parent_kind,
                           name, description, user_id,
                           uploaded_at, changed_at
                    FROM folder
                    WHERE parent_kind = ? AND parent_id = ?
                    ORDER BY name ASC, folder_id ASC
                    LIMIT ?
                    "#,
                    parent_kind,
                    parent_id,
                    limit,
                )
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|r| {
                    Ok(ListChildrenByParentFolderView {
                        id: uuid::Uuid::from_slice(&r.folder_id)?,
                        parent_id: uuid::Uuid::from_slice(&r.parent_id)?,
                        parent_kind: r.parent_kind,
                        name: r.name,
                        description: r.description,
                        user_id: uuid::Uuid::from_slice(&r.user_id)?,
                        uploaded_at: r.uploaded_at.and_utc(),
                        changed_at: r.changed_at.and_utc(),
                    })
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?,
            };

            for f in folders {
                items.push(ListChildrenByParentView::Folder(f));
            }
        }

        let remaining = limit - items.len() as i64;
        if remaining > 0 {
            let files: Vec<ListChildrenByParentFileView> = match file_cursor {
                Some((cursor_name, cursor_id)) => sqlx::query!(
                    r#"
                    SELECT file_id, name, description, user_id,
                           storage_id, file_size,
                           parent_id, parent_kind,
                           uploaded_at, changed_at
                    FROM file
                    WHERE parent_kind = ? AND parent_id = ?
                      AND (name, file_id) > (?, ?)
                    ORDER BY name ASC, file_id ASC
                    LIMIT ?
                    "#,
                    parent_kind,
                    parent_id,
                    cursor_name,
                    cursor_id,
                    remaining,
                )
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|r| {
                    Ok(ListChildrenByParentFileView {
                        id: uuid::Uuid::from_slice(&r.file_id)?,
                        name: r.name,
                        description: r.description,
                        user_id: uuid::Uuid::from_slice(&r.user_id)?,
                        storage_id: uuid::Uuid::from_slice(&r.storage_id)?,
                        file_size: r.file_size,
                        parent_id: uuid::Uuid::from_slice(&r.parent_id)?,
                        parent_kind: r.parent_kind,
                        uploaded_at: r.uploaded_at.and_utc(),
                        changed_at: r.changed_at.and_utc(),
                    })
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?,
                None => sqlx::query!(
                    r#"
                    SELECT file_id, name, description, user_id,
                           storage_id, file_size,
                           parent_id, parent_kind,
                           uploaded_at, changed_at
                    FROM file
                    WHERE parent_kind = ? AND parent_id = ?
                    ORDER BY name ASC, file_id ASC
                    LIMIT ?
                    "#,
                    parent_kind,
                    parent_id,
                    remaining,
                )
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|r| {
                    Ok(ListChildrenByParentFileView {
                        id: uuid::Uuid::from_slice(&r.file_id)?,
                        name: r.name,
                        description: r.description,
                        user_id: uuid::Uuid::from_slice(&r.user_id)?,
                        storage_id: uuid::Uuid::from_slice(&r.storage_id)?,
                        file_size: r.file_size,
                        parent_id: uuid::Uuid::from_slice(&r.parent_id)?,
                        parent_kind: r.parent_kind,
                        uploaded_at: r.uploaded_at.and_utc(),
                        changed_at: r.changed_at.and_utc(),
                    })
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?,
            };

            for f in files {
                items.push(ListChildrenByParentView::File(f));
            }
        }

        Ok(items)
    }

    async fn get_file(&self, file_id: uuid::Uuid) -> Result<Option<GetFileView>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT file_id, name, description, user_id,
                   storage_type, storage_id, file_size,
                   parent_id, parent_kind,
                   uploaded_at, changed_at
            FROM file
            WHERE file_id = ?
            "#,
            file_id.as_bytes().as_slice(),
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(GetFileView {
                id: uuid::Uuid::from_slice(&r.file_id)?,
                name: r.name,
                description: r.description,
                user_id: uuid::Uuid::from_slice(&r.user_id)?,
                storage_type: r.storage_type,
                storage_id: uuid::Uuid::from_slice(&r.storage_id)?,
                file_size: r.file_size,
                parent_id: uuid::Uuid::from_slice(&r.parent_id)?,
                parent_kind: r.parent_kind,
                uploaded_at: r.uploaded_at.and_utc(),
                changed_at: r.changed_at.and_utc(),
            })),
            None => Ok(None),
        }
    }

    async fn get_folder(
        &self,
        folder_id: uuid::Uuid,
    ) -> Result<Option<GetFolderView>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT folder_id, parent_id, parent_kind, depth,
                   name, description, user_id,
                   uploaded_at, changed_at
            FROM folder
            WHERE folder_id = ?
            "#,
            folder_id.as_bytes().as_slice(),
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(GetFolderView {
                id: uuid::Uuid::from_slice(&r.folder_id)?,
                parent_id: uuid::Uuid::from_slice(&r.parent_id)?,
                parent_kind: r.parent_kind,
                depth: r.depth,
                name: r.name,
                description: r.description,
                user_id: uuid::Uuid::from_slice(&r.user_id)?,
                uploaded_at: r.uploaded_at.and_utc(),
                changed_at: r.changed_at.and_utc(),
            })),
            None => Ok(None),
        }
    }

    async fn get_text_content(
        &self,
        storage_id: uuid::Uuid,
    ) -> Result<Option<GetTextContentView>, anyhow::Error> {
        let row = sqlx::query!(
            "SELECT content FROM text_storage WHERE storage_id = ?",
            storage_id.as_bytes().as_slice(),
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| GetTextContentView { content: r.content }))
    }
}
