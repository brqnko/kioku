#[async_trait::async_trait]
pub trait FileRepository<C>: Send + Sync {
    async fn find_for_update(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::File>, anyhow::Error>;
    async fn find_by_parent_for_update(
        &self,
        c: &mut C,
        parent: &super::domain::ParentId,
    ) -> Result<Vec<super::domain::File>, anyhow::Error>;
    async fn save(
        &self,
        c: &mut C,
        file: &super::domain::File,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct FileRepositoryImpl {}

impl Default for FileRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl FileRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[allow(clippy::too_many_arguments)]
fn row_to_file(
    file_id: Vec<u8>,
    name: String,
    description: String,
    user_id: Vec<u8>,
    storage_type: u8,
    storage_id: Vec<u8>,
    file_size: u64,
    parent_id: Vec<u8>,
    parent_kind: u8,
    uploaded_at: chrono::NaiveDateTime,
    changed_at: chrono::NaiveDateTime,
) -> Result<super::domain::File, anyhow::Error> {
    let parent =
        super::domain::ParentId::from_raw(uuid::Uuid::from_slice(&parent_id)?, parent_kind)
            .map_err(|e| anyhow::anyhow!("invalid parent: {:?}", e))?;
    let storage_type = super::domain::StorageType::try_from(storage_type)?;
    Ok(super::domain::File {
        id: uuid::Uuid::from_slice(&file_id)?,
        name: super::domain::FileName(name),
        description: super::domain::FileDescription(description),
        user_id: uuid::Uuid::from_slice(&user_id)?,
        storage_type,
        storage_id: uuid::Uuid::from_slice(&storage_id)?,
        file_size,
        parent,
        uploaded_at: uploaded_at.and_utc(),
        changed_at: changed_at.and_utc(),
    })
}

#[async_trait::async_trait]
impl FileRepository<sqlx::MySqlConnection> for FileRepositoryImpl {
    async fn find_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::File>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT file_id, name, description, user_id,
                   storage_type, storage_id, file_size,
                   parent_id, parent_kind,
                   uploaded_at, changed_at
            FROM file
            WHERE file_id = ?
            LIMIT 1
            FOR UPDATE
            "#,
            id.as_bytes().as_slice(),
        )
        .fetch_optional(c)
        .await?;

        match row {
            Some(r) => Ok(Some(row_to_file(
                r.file_id,
                r.name,
                r.description,
                r.user_id,
                r.storage_type,
                r.storage_id,
                r.file_size,
                r.parent_id,
                r.parent_kind,
                r.uploaded_at,
                r.changed_at,
            )?)),
            None => Ok(None),
        }
    }

    async fn find_by_parent_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        parent: &super::domain::ParentId,
    ) -> Result<Vec<super::domain::File>, anyhow::Error> {
        let parent_kind = parent.kind();
        let parent_uuid = parent.id();
        let parent_id_bytes = parent_uuid.as_bytes().as_slice();
        let rows = sqlx::query!(
            r#"
            SELECT file_id, name, description, user_id,
                   storage_type, storage_id, file_size,
                   parent_id, parent_kind,
                   uploaded_at, changed_at
            FROM file
            WHERE parent_kind = ? AND parent_id = ?
            FOR UPDATE
            "#,
            parent_kind,
            parent_id_bytes,
        )
        .fetch_all(c)
        .await?;

        rows.into_iter()
            .map(|r| {
                row_to_file(
                    r.file_id,
                    r.name,
                    r.description,
                    r.user_id,
                    r.storage_type,
                    r.storage_id,
                    r.file_size,
                    r.parent_id,
                    r.parent_kind,
                    r.uploaded_at,
                    r.changed_at,
                )
            })
            .collect::<Result<Vec<super::domain::File>, anyhow::Error>>()
    }

    async fn save(
        &self,
        c: &mut sqlx::MySqlConnection,
        file: &super::domain::File,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        let storage_type = match file.storage_type {
            super::domain::StorageType::Object => 0u8,
            super::domain::StorageType::Text => 1u8,
        };
        let parent_uuid = file.parent.id();
        let parent_id_bytes = parent_uuid.as_bytes().as_slice();
        let parent_kind = file.parent.kind();
        sqlx::query!(
            r#"
            INSERT INTO file
                (file_id, name, description, user_id,
                 storage_type, storage_id, file_size,
                 parent_id, parent_kind,
                 uploaded_at, changed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                name         = VALUES(name),
                description  = VALUES(description),
                user_id      = VALUES(user_id),
                storage_type = VALUES(storage_type),
                storage_id   = VALUES(storage_id),
                file_size    = VALUES(file_size),
                parent_id    = VALUES(parent_id),
                parent_kind  = VALUES(parent_kind),
                changed_at   = VALUES(changed_at)
            "#,
            file.id.as_bytes().as_slice(),
            file.name.0,
            file.description.0,
            file.user_id.as_bytes().as_slice(),
            storage_type,
            file.storage_id.as_bytes().as_slice(),
            file.file_size,
            parent_id_bytes,
            parent_kind,
            file.uploaded_at,
            file.changed_at,
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
            "DELETE FROM file WHERE file_id = ?",
            id.as_bytes().as_slice(),
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }
}

#[async_trait::async_trait]
pub trait FolderRepository<C>: Send + Sync {
    async fn find_for_update(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::Folder>, anyhow::Error>;
    async fn save(
        &self,
        c: &mut C,
        folder: &super::domain::Folder,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct FolderRepositoryImpl {}

impl Default for FolderRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl FolderRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl FolderRepository<sqlx::MySqlConnection> for FolderRepositoryImpl {
    async fn find_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::Folder>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT folder_id, parent_id, parent_kind, depth,
                   name, description, user_id,
                   uploaded_at, changed_at
            FROM folder
            WHERE folder_id = ?
            LIMIT 1
            FOR UPDATE
            "#,
            id.as_bytes().as_slice(),
        )
        .fetch_optional(c)
        .await?;

        match row {
            Some(r) => {
                let parent = super::domain::ParentId::from_raw(
                    uuid::Uuid::from_slice(&r.parent_id)?,
                    r.parent_kind,
                )
                .map_err(|e| anyhow::anyhow!("invalid parent: {:?}", e))?;
                Ok(Some(super::domain::Folder {
                    id: uuid::Uuid::from_slice(&r.folder_id)?,
                    parent,
                    depth: r.depth,
                    name: super::domain::FolderName(r.name),
                    description: super::domain::FolderDescription(r.description),
                    user_id: uuid::Uuid::from_slice(&r.user_id)?,
                    uploaded_at: r.uploaded_at.and_utc(),
                    changed_at: r.changed_at.and_utc(),
                }))
            }
            None => Ok(None),
        }
    }

    async fn save(
        &self,
        c: &mut sqlx::MySqlConnection,
        folder: &super::domain::Folder,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        let parent_uuid = folder.parent.id();
        let parent_id_bytes = parent_uuid.as_bytes().as_slice();
        let parent_kind = folder.parent.kind();
        sqlx::query!(
            r#"
            INSERT INTO folder
                (folder_id, parent_id, parent_kind, depth,
                 name, description, user_id,
                 uploaded_at, changed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                parent_id   = VALUES(parent_id),
                parent_kind = VALUES(parent_kind),
                depth       = VALUES(depth),
                name        = VALUES(name),
                description = VALUES(description),
                changed_at  = VALUES(changed_at)
            "#,
            folder.id.as_bytes().as_slice(),
            parent_id_bytes,
            parent_kind,
            folder.depth,
            folder.name.0,
            folder.description.0,
            folder.user_id.as_bytes().as_slice(),
            folder.uploaded_at,
            folder.changed_at,
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
            "DELETE FROM folder WHERE folder_id = ?",
            id.as_bytes().as_slice(),
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }
}

#[async_trait::async_trait]
pub trait TextStorageRepository<C>: Send + Sync {
    async fn find_for_update(
        &self,
        c: &mut C,
        storage_id: uuid::Uuid,
    ) -> Result<Option<super::domain::Text>, anyhow::Error>;
    async fn save(
        &self,
        c: &mut C,
        storage_id: uuid::Uuid,
        text: &super::domain::Text,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(
        &self,
        c: &mut C,
        storage_id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct TextStorageRepositoryImpl {}

impl Default for TextStorageRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl TextStorageRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl TextStorageRepository<sqlx::MySqlConnection> for TextStorageRepositoryImpl {
    async fn find_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        storage_id: uuid::Uuid,
    ) -> Result<Option<super::domain::Text>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT content
            FROM text_storage
            WHERE storage_id = ?
            LIMIT 1
            FOR UPDATE
            "#,
            storage_id.as_bytes().as_slice(),
        )
        .fetch_optional(c)
        .await?;

        Ok(row.map(|r| super::domain::Text(r.content)))
    }

    async fn save(
        &self,
        c: &mut sqlx::MySqlConnection,
        storage_id: uuid::Uuid,
        text: &super::domain::Text,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        sqlx::query!(
            r#"
            INSERT INTO text_storage (storage_id, content)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE
                content = VALUES(content)
            "#,
            storage_id.as_bytes().as_slice(),
            text.0,
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }

    async fn remove(
        &self,
        c: &mut sqlx::MySqlConnection,
        storage_id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        sqlx::query!(
            "DELETE FROM text_storage WHERE storage_id = ?",
            storage_id.as_bytes().as_slice(),
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }
}

#[async_trait::async_trait]
pub trait FileEmbeddingRepository<C>: Send + Sync {
    async fn save(
        &self,
        c: &mut C,
        file_embedding: &super::domain::FileEmbedding,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn delete_all_by_file_id(
        &self,
        c: &mut C,
        file_id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct FileEmbeddingRepositoryImpl {
    mysql_kind: crate::util::dialect::MySQLKind,
}

impl FileEmbeddingRepositoryImpl {
    pub fn new(mysql_kind: crate::util::dialect::MySQLKind) -> Self {
        Self { mysql_kind }
    }
}

fn format_embedding(values: &[f32]) -> String {
    let mut s = String::with_capacity(values.len() * 12 + 2);
    s.push('[');
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&v.to_string());
    }
    s.push(']');
    s
}

#[async_trait::async_trait]
impl FileEmbeddingRepository<sqlx::MySqlConnection> for FileEmbeddingRepositoryImpl {
    async fn save(
        &self,
        c: &mut sqlx::MySqlConnection,
        file_embedding: &super::domain::FileEmbedding,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        let embedding = format_embedding(&file_embedding.embedding.0);
        let vector_expr = match self.mysql_kind {
            crate::util::dialect::MySQLKind::MariaDB => "VEC_FromText(?)",
            crate::util::dialect::MySQLKind::TiDB => "CAST(? AS VECTOR(1024))",
        };
        let sql = format!(
            r#"
            INSERT INTO file_embedding
                (file_embedding_id, file_id, original_text, embedding, indexed_at)
            VALUES (?, ?, ?, {}, ?)
            "#,
            vector_expr,
        );
        sqlx::query(&sql)
            .bind(file_embedding.id.as_bytes().as_slice())
            .bind(file_embedding.file_id.as_bytes().as_slice())
            .bind(&file_embedding.original_text.0)
            .bind(&embedding)
            .bind(file_embedding.indexed_at)
            .execute(c)
            .await?;

        Ok(Ok(()))
    }

    async fn delete_all_by_file_id(
        &self,
        c: &mut sqlx::MySqlConnection,
        file_id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        sqlx::query!(
            "DELETE FROM file_embedding WHERE file_id = ?",
            file_id.as_bytes().as_slice(),
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }
}
