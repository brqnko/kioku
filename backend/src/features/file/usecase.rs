// request upload url

pub struct RequestUploadUrlInput {
    pub user_id: uuid::Uuid,
    pub content_type: String,
    pub content_length: i64,
}

pub struct RequestUploadUrlOutput {
    pub storage_id: uuid::Uuid,
    pub url: String,
    pub method: String,
    pub content_type: String,
    pub content_length: i64,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub async fn request_upload_url(
    app: &crate::app::App,
    input: RequestUploadUrlInput,
) -> Result<Result<RequestUploadUrlOutput, crate::domain::DomainError>, anyhow::Error> {
    let content_type = match super::domain::ContentType::from_mime(&input.content_type) {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };
    let content_length = match super::domain::ContentLength::new(input.content_length) {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    let mut tx = app.pool.begin().await?;

    let mut user = match app
        .user_repository
        .find_for_update(&mut tx, input.user_id)
        .await?
    {
        Some(u) => u,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "user_not_found",
                "user not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if let Err(err) = user.check_file_upload_daily_quota()? {
        return Ok(Err(err));
    }

    let storage_id = uuid::Uuid::new_v4();
    let expires_in = std::time::Duration::from_secs(60);

    let presigned = app
        .temporary_storage_service
        .presign_put(
            storage_id,
            content_type.as_mime(),
            content_length.0,
            expires_in,
        )
        .await?;

    user.consume_file_upload_daily_quota();
    match app.user_repository.save(&mut tx, &user).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(RequestUploadUrlOutput {
        storage_id,
        url: presigned.url,
        method: presigned.method,
        content_type: presigned.content_type,
        content_length: presigned.content_length,
        expires_at: presigned.expires_at,
    }))
}

// create file

pub struct CreateFileInput {
    pub user_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub storage_id: Option<uuid::Uuid>,
    pub text: Option<String>,
    pub parent_id: uuid::Uuid,
    pub parent_kind: u8,
}

pub struct CreateFileOutput {
    pub file: super::domain::File,
}

pub async fn create_file(
    app: &crate::app::App,
    input: CreateFileInput,
) -> Result<Result<CreateFileOutput, crate::domain::DomainError>, anyhow::Error> {
    let parent = match super::domain::ParentId::from_raw(input.parent_id, input.parent_kind) {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    let storage_id = uuid::Uuid::new_v4();
    let mut pending_put: Option<(uuid::Uuid, String, Vec<u8>)> = None;
    let mut text_to_persist: Option<super::domain::Text> = None;

    let (storage_type, file_size) = match (input.storage_id, input.text) {
        (Some(temp_id), None) => {
            let got = app.temporary_storage_service.get_object(temp_id).await?;
            match super::domain::ContentLength::new(got.content_length) {
                Ok(_) => {}
                Err(err) => return Ok(Err(err)),
            }
            let size = got.content_length as u64;
            pending_put = Some((temp_id, got.content_type, got.body));
            (super::domain::StorageType::Object, size)
        }
        (None, Some(text)) => {
            let text_vo = match super::domain::Text::new(text) {
                Ok(ok) => ok,
                Err(err) => return Ok(Err(err)),
            };
            let size = text_vo.0.len() as u64;
            text_to_persist = Some(text_vo);
            (super::domain::StorageType::Text, size)
        }
        _ => {
            return Ok(Err(crate::domain::DomainError::new(
                "invalid_input",
                "exactly one of storage_id or text must be provided".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            )));
        }
    };

    let mut tx = app.pool.begin().await?;

    match &parent {
        super::domain::ParentId::Project(project_id) => {
            match app
                .project_repository
                .find_for_update(&mut tx, *project_id)
                .await?
            {
                Some(project) => {
                    if project.created_by != input.user_id {
                        return Ok(Err(crate::domain::DomainError::new(
                            "forbidden",
                            "project does not belong to the user".to_string(),
                            crate::domain::DomainErrorKind::Forbidden,
                        )));
                    }
                }
                None => {
                    return Ok(Err(crate::domain::DomainError::new(
                        "project_not_found",
                        "project not found".to_string(),
                        crate::domain::DomainErrorKind::NotFound,
                    )));
                }
            }
        }
        super::domain::ParentId::Folder(folder_id) => {
            match app
                .folder_repository
                .find_for_update(&mut tx, *folder_id)
                .await?
            {
                Some(folder) => {
                    if folder.user_id != input.user_id {
                        return Ok(Err(crate::domain::DomainError::new(
                            "forbidden",
                            "folder does not belong to the user".to_string(),
                            crate::domain::DomainErrorKind::Forbidden,
                        )));
                    }
                }
                None => {
                    return Ok(Err(crate::domain::DomainError::new(
                        "folder_not_found",
                        "folder not found".to_string(),
                        crate::domain::DomainErrorKind::NotFound,
                    )));
                }
            }
        }
    }

    if let Some(text_vo) = text_to_persist {
        match app
            .text_storage_repository
            .save(&mut tx, storage_id, &text_vo)
            .await?
        {
            Ok(()) => {}
            Err(err) => return Ok(Err(err)),
        }
    }

    let file = match super::domain::File::new(
        input.user_id,
        input.name,
        input.description,
        storage_type,
        storage_id,
        file_size,
        parent,
        super::domain::FileOption::default(),
    )? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    match app.file_repository.save(&mut tx, &file).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }

    let temp_to_cleanup = match pending_put {
        Some((temp_id, content_type, body)) => {
            app.storage_service
                .put_object(storage_id, &content_type, body)
                .await?;
            Some(temp_id)
        }
        None => None,
    };

    if let Err(err) = tx.commit().await {
        if temp_to_cleanup.is_some() {
            let _ = app.storage_service.delete(storage_id).await;
        }
        return Err(err.into());
    }

    if let Some(temp_id) = temp_to_cleanup {
        let _ = app.temporary_storage_service.delete(temp_id).await;
    }

    Ok(Ok(CreateFileOutput { file }))
}

// update file

pub struct UpdateFileInput {
    pub user_id: uuid::Uuid,
    pub file_id: uuid::Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct UpdateFileOutput {
    pub file: super::domain::File,
}

pub async fn update_file(
    app: &crate::app::App,
    input: UpdateFileInput,
) -> Result<Result<UpdateFileOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let mut file = match app
        .file_repository
        .find_for_update(&mut tx, input.file_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "file_not_found",
                "file not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if file.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "file does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    if let Some(name) = input.name {
        match file.set_name(name) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        }
    }

    if let Some(description) = input.description {
        match file.set_description(description) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        }
    }

    match app.file_repository.save(&mut tx, &file).await? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(UpdateFileOutput { file }))
}

// update file text

pub struct UpdateFileTextInput {
    pub user_id: uuid::Uuid,
    pub file_id: uuid::Uuid,
    pub text: String,
}

pub struct UpdateFileTextOutput {
    pub file: super::domain::File,
}

pub async fn update_file_text(
    app: &crate::app::App,
    input: UpdateFileTextInput,
) -> Result<Result<UpdateFileTextOutput, crate::domain::DomainError>, anyhow::Error> {
    let text = match super::domain::Text::new(input.text) {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    let mut tx = app.pool.begin().await?;

    let mut file = match app
        .file_repository
        .find_for_update(&mut tx, input.file_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "file_not_found",
                "file not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if file.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "file does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    match file.set_text_content(&text) {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }

    match app
        .text_storage_repository
        .save(&mut tx, file.storage_id, &text)
        .await?
    {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }

    match app.file_repository.save(&mut tx, &file).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(UpdateFileTextOutput { file }))
}

// get file content

// get file content

pub struct GetFileContentInput {
    pub user_id: uuid::Uuid,
    pub file_id: uuid::Uuid,
}

pub struct GetFileContentOutput {
    pub file: super::query_service::GetFileView,
    pub content: FileContent,
}

pub enum FileContent {
    Url {
        url: String,
        method: String,
        expires_at: chrono::DateTime<chrono::Utc>,
    },
    Text {
        content: String,
    },
}

pub async fn get_file_content(
    app: &crate::app::App,
    input: GetFileContentInput,
) -> Result<Result<GetFileContentOutput, crate::domain::DomainError>, anyhow::Error> {
    let file = match app.file_query_service.get_file(input.file_id).await? {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "file_not_found",
                "file not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if file.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "file does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    let storage_type = super::domain::StorageType::try_from(file.storage_type)?;

    let content = match storage_type {
        super::domain::StorageType::Object => {
            let presigned = app
                .storage_service
                .presign_get(file.storage_id, std::time::Duration::from_secs(60 * 60))
                .await?;
            FileContent::Url {
                url: presigned.url,
                method: presigned.method,
                expires_at: presigned.expires_at,
            }
        }
        super::domain::StorageType::Text => {
            let text = match app
                .file_query_service
                .get_text_content(file.storage_id)
                .await?
            {
                Some(ok) => ok,
                None => {
                    return Err(anyhow::anyhow!(
                        "text storage missing for file: {}",
                        file.id
                    ));
                }
            };
            FileContent::Text {
                content: text.content,
            }
        }
    };

    let mut tx = app.pool.begin().await?;

    let mut user = match app
        .user_repository
        .find_for_update(&mut tx, input.user_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Err(anyhow::anyhow!("user not found: {}", input.user_id));
        }
    };

    match user.push_recent_seen_file_id(file.id) {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }

    match app.user_repository.save(&mut tx, &user).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(GetFileContentOutput { file, content }))
}

// get file raw url

pub struct GetFileRawUrlInput {
    pub user_id: uuid::Uuid,
    pub file_id: uuid::Uuid,
}

pub struct GetFileRawUrlOutput {
    pub url: String,
}

pub async fn get_file_raw_url(
    app: &crate::app::App,
    input: GetFileRawUrlInput,
) -> Result<Result<GetFileRawUrlOutput, crate::domain::DomainError>, anyhow::Error> {
    let file = match app.file_query_service.get_file(input.file_id).await? {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "file_not_found",
                "file not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if file.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "file does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    let storage_type = super::domain::StorageType::try_from(file.storage_type)?;
    match storage_type {
        super::domain::StorageType::Object => {}
        super::domain::StorageType::Text => {
            return Ok(Err(crate::domain::DomainError::new(
                "invalid_storage_type",
                "raw fetch is only supported for object storage".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            )));
        }
    }

    let presigned = app
        .storage_service
        .presign_get(file.storage_id, std::time::Duration::from_secs(60 * 60))
        .await?;

    Ok(Ok(GetFileRawUrlOutput {
        url: presigned.url,
    }))
}

// index file

pub struct IndexFileInput {
    pub file_id: uuid::Uuid,
}

pub struct IndexFileOutput {}

pub async fn index_file(
    app: &crate::app::App,
    input: IndexFileInput,
) -> Result<Result<IndexFileOutput, crate::domain::DomainError>, anyhow::Error> {
    let file = match app.file_query_service.get_file(input.file_id).await? {
        Some(ok) => ok,
        None => return Ok(Ok(IndexFileOutput {})),
    };

    let storage_type = super::domain::StorageType::try_from(file.storage_type)?;

    match storage_type {
        super::domain::StorageType::Text => {
            let text = match app
                .file_query_service
                .get_text_content(file.storage_id)
                .await?
            {
                Some(ok) => ok,
                None => {
                    return Err(anyhow::anyhow!(
                        "text storage missing for file: {}",
                        file.id
                    ));
                }
            };

            let chunks = chunk_text(&text.content, super::domain::OriginalText::MAX_BYTES);
            if chunks.is_empty() {
                return Ok(Ok(IndexFileOutput {}));
            }

            let mut chunks_by_id = std::collections::HashMap::<uuid::Uuid, String>::new();
            for chunk in chunks {
                chunks_by_id.insert(uuid::Uuid::new_v4(), chunk);
            }

            let embeddings = app.embedding_client.embed(chunks_by_id.clone()).await?;

            let mut file_embeddings = Vec::new();
            for (id, vector) in embeddings {
                let original_text = match chunks_by_id.remove(&id) {
                    Some(ok) => ok,
                    None => {
                        return Err(anyhow::anyhow!(
                            "embedding result has unknown chunk id: {id}"
                        ));
                    }
                };
                let fe = match super::domain::FileEmbedding::new(
                    file.id,
                    original_text,
                    vector,
                    super::domain::FileEmbeddingOption::default(),
                )? {
                    Ok(ok) => ok,
                    Err(err) => return Ok(Err(err)),
                };
                file_embeddings.push(fe);
            }

            let mut tx = app.pool.begin().await?;

            if app
                .file_repository
                .find_for_update(&mut tx, file.id)
                .await?
                .is_none()
            {
                return Ok(Ok(IndexFileOutput {}));
            }

            match app
                .file_embedding_repository
                .delete_all_by_file_id(&mut tx, file.id)
                .await?
            {
                Ok(()) => {}
                Err(err) => return Ok(Err(err)),
            }

            for fe in &file_embeddings {
                match app.file_embedding_repository.save(&mut tx, fe).await? {
                    Ok(()) => {}
                    Err(err) => return Ok(Err(err)),
                }
            }

            tx.commit().await?;

            Ok(Ok(IndexFileOutput {}))
        }
        super::domain::StorageType::Object => {
            todo!("object indexing not implemented yet")
        }
    }
}

fn chunk_text(text: &str, max_bytes: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut buf = String::new();
    for ch in text.chars() {
        if !buf.is_empty() && buf.len() + ch.len_utf8() > max_bytes {
            chunks.push(std::mem::take(&mut buf));
        }
        buf.push(ch);
    }
    if !buf.is_empty() {
        chunks.push(buf);
    }
    chunks
}

// remove file

pub struct RemoveFileInput {
    pub user_id: uuid::Uuid,
    pub file_id: uuid::Uuid,
}

pub struct RemoveFileOutput {}

pub async fn remove_file(
    app: &crate::app::App,
    input: RemoveFileInput,
) -> Result<Result<RemoveFileOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let file = match app
        .file_repository
        .find_for_update(&mut tx, input.file_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "file_not_found",
                "file not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if file.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "file does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    if let super::domain::StorageType::Text = file.storage_type {
        match app
            .text_storage_repository
            .remove(&mut tx, file.storage_id)
            .await?
        {
            Ok(()) => {}
            Err(err) => return Ok(Err(err)),
        }
    }

    match app.file_repository.remove(&mut tx, file.id).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    if let super::domain::StorageType::Object = file.storage_type {
        let _ = app.storage_service.delete(file.storage_id).await;
    }

    Ok(Ok(RemoveFileOutput {}))
}

// create folder

pub struct CreateFolderInput {
    pub user_id: uuid::Uuid,
    pub parent_id: uuid::Uuid,
    pub parent_kind: u8,
    pub name: String,
    pub description: String,
}

pub struct CreateFolderOutput {
    pub folder: super::domain::Folder,
}

pub async fn create_folder(
    app: &crate::app::App,
    input: CreateFolderInput,
) -> Result<Result<CreateFolderOutput, crate::domain::DomainError>, anyhow::Error> {
    let parent = match super::domain::ParentId::from_raw(input.parent_id, input.parent_kind) {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    let mut tx = app.pool.begin().await?;

    let depth = match &parent {
        super::domain::ParentId::Project(project_id) => {
            match app
                .project_repository
                .find_for_update(&mut tx, *project_id)
                .await?
            {
                Some(project) => {
                    if project.created_by != input.user_id {
                        return Ok(Err(crate::domain::DomainError::new(
                            "forbidden",
                            "project does not belong to the user".to_string(),
                            crate::domain::DomainErrorKind::Forbidden,
                        )));
                    }
                }
                None => {
                    return Ok(Err(crate::domain::DomainError::new(
                        "project_not_found",
                        "project not found".to_string(),
                        crate::domain::DomainErrorKind::NotFound,
                    )));
                }
            }
            0
        }
        super::domain::ParentId::Folder(folder_id) => {
            match app
                .folder_repository
                .find_for_update(&mut tx, *folder_id)
                .await?
            {
                Some(parent_folder) => {
                    if parent_folder.user_id != input.user_id {
                        return Ok(Err(crate::domain::DomainError::new(
                            "forbidden",
                            "folder does not belong to the user".to_string(),
                            crate::domain::DomainErrorKind::Forbidden,
                        )));
                    }
                    parent_folder.depth + 1
                }
                None => {
                    return Ok(Err(crate::domain::DomainError::new(
                        "folder_not_found",
                        "folder not found".to_string(),
                        crate::domain::DomainErrorKind::NotFound,
                    )));
                }
            }
        }
    };

    let folder = match super::domain::Folder::new(
        input.user_id,
        parent,
        depth,
        input.name,
        input.description,
        super::domain::FolderOption {
            ..Default::default()
        },
    )? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    match app.folder_repository.save(&mut tx, &folder).await? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    tx.commit().await?;

    Ok(Ok(CreateFolderOutput { folder }))
}

// get folder

pub struct GetFolderInput {
    pub user_id: uuid::Uuid,
    pub folder_id: uuid::Uuid,
}

pub struct GetFolderOutput {
    pub folder: super::query_service::GetFolderView,
}

pub async fn get_folder(
    app: &crate::app::App,
    input: GetFolderInput,
) -> Result<Result<GetFolderOutput, crate::domain::DomainError>, anyhow::Error> {
    let folder = match app.file_query_service.get_folder(input.folder_id).await? {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "folder_not_found",
                "folder not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if folder.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "folder does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    Ok(Ok(GetFolderOutput { folder }))
}

// update folder

pub struct UpdateFolderInput {
    pub user_id: uuid::Uuid,
    pub folder_id: uuid::Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct UpdateFolderOutput {
    pub folder: super::domain::Folder,
}

pub async fn update_folder(
    app: &crate::app::App,
    input: UpdateFolderInput,
) -> Result<Result<UpdateFolderOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let mut folder = match app
        .folder_repository
        .find_for_update(&mut tx, input.folder_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "folder_not_found",
                "folder not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if folder.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "folder does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    if let Some(name) = input.name {
        match folder.set_name(name) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        }
    }

    if let Some(description) = input.description {
        match folder.set_description(description) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        }
    }

    match app.folder_repository.save(&mut tx, &folder).await? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(UpdateFolderOutput { folder }))
}

// remove folder

pub struct RemoveFolderInput {
    pub user_id: uuid::Uuid,
    pub folder_id: uuid::Uuid,
}

pub struct RemoveFolderOutput {}

pub async fn remove_folder(
    app: &crate::app::App,
    input: RemoveFolderInput,
) -> Result<Result<RemoveFolderOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let folder = match app
        .folder_repository
        .find_for_update(&mut tx, input.folder_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "folder_not_found",
                "folder not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if folder.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "folder does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    if app.file_query_service.folder_has_child(folder.id).await? {
        return Ok(Err(crate::domain::DomainError::new(
            "folder_not_empty",
            "folder has children".to_string(),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }

    match app.folder_repository.remove(&mut tx, folder.id).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(RemoveFolderOutput {}))
}

// get folder ancestors

pub struct GetFolderAncestorsInput {
    pub user_id: uuid::Uuid,
    pub folder_id: uuid::Uuid,
}

pub struct GetFolderAncestorsOutput {
    pub ancestors: Vec<super::query_service::AncestorView>,
}

pub async fn get_folder_ancestors(
    app: &crate::app::App,
    input: GetFolderAncestorsInput,
) -> Result<Result<GetFolderAncestorsOutput, crate::domain::DomainError>, anyhow::Error> {
    let folder = match app.file_query_service.get_folder(input.folder_id).await? {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "folder_not_found",
                "folder not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if folder.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "folder does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    let ancestors = app
        .file_query_service
        .list_ancestors(input.user_id, folder.parent_id, folder.parent_kind)
        .await?;

    Ok(Ok(GetFolderAncestorsOutput { ancestors }))
}

// get file ancestors

pub struct GetFileAncestorsInput {
    pub user_id: uuid::Uuid,
    pub file_id: uuid::Uuid,
}

pub struct GetFileAncestorsOutput {
    pub ancestors: Vec<super::query_service::AncestorView>,
}

pub async fn get_file_ancestors(
    app: &crate::app::App,
    input: GetFileAncestorsInput,
) -> Result<Result<GetFileAncestorsOutput, crate::domain::DomainError>, anyhow::Error> {
    let file = match app.file_query_service.get_file(input.file_id).await? {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "file_not_found",
                "file not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if file.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "file does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    let ancestors = app
        .file_query_service
        .list_ancestors(input.user_id, file.parent_id, file.parent_kind)
        .await?;

    Ok(Ok(GetFileAncestorsOutput { ancestors }))
}

// list children

pub struct ListChildrenInput {
    pub user_id: uuid::Uuid,
    pub parent_id: uuid::Uuid,
    pub parent_kind: u8,
    pub cursor: Option<super::query_service::ListChildrenByParentCursor>,
    pub limit: u32,
}

pub struct ListChildrenOutput {
    pub items: Vec<super::query_service::ListChildrenByParentView>,
    pub next_cursor: Option<super::query_service::ListChildrenByParentCursor>,
}

pub async fn list_children(
    app: &crate::app::App,
    input: ListChildrenInput,
) -> Result<Result<ListChildrenOutput, crate::domain::DomainError>, anyhow::Error> {
    if input.limit > 32 {
        return Ok(Err(crate::domain::DomainError::new(
            "invalid_limit",
            "limit must be 32 or less".to_string(),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }

    let parent = match super::domain::ParentId::from_raw(input.parent_id, input.parent_kind) {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    let mut rows = app
        .file_query_service
        .list_children_by_parent(&parent, input.cursor, input.limit + 1)
        .await?
        .into_iter()
        .filter(|item| {
            let owner = match item {
                super::query_service::ListChildrenByParentView::Folder(f) => f.user_id,
                super::query_service::ListChildrenByParentView::File(f) => f.user_id,
            };
            owner == input.user_id
        })
        .collect::<Vec<super::query_service::ListChildrenByParentView>>();

    let next_cursor = if rows.len() as u32 > input.limit {
        rows.pop().map(|item| match item {
            super::query_service::ListChildrenByParentView::Folder(f) => {
                super::query_service::ListChildrenByParentCursor {
                    phase: super::query_service::ListChildrenByParentPhase::Folders,
                    name: f.name,
                    id: f.id,
                }
            }
            super::query_service::ListChildrenByParentView::File(f) => {
                super::query_service::ListChildrenByParentCursor {
                    phase: super::query_service::ListChildrenByParentPhase::Files,
                    name: f.name,
                    id: f.id,
                }
            }
        })
    } else {
        None
    };

    Ok(Ok(ListChildrenOutput {
        items: rows,
        next_cursor,
    }))
}

pub struct RunCodeInput {
    pub code: String,
    pub compiler: String,
    pub stdin: Option<String>,
    pub compiler_options: Option<String>,
    pub compiler_option_raw: Option<String>,
    pub runtime_option_raw: Option<String>,
}

pub struct RunCodeOutput {
    pub status: Option<String>,
    pub signal: Option<String>,
    pub compiler_output: Option<String>,
    pub compiler_error: Option<String>,
    pub compiler_message: Option<String>,
    pub program_output: Option<String>,
    pub program_error: Option<String>,
    pub program_message: Option<String>,
}

fn is_valid_compiler_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= crate::util::code_runner::MAX_COMPILER_LEN
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '+'))
}

pub async fn run_code(
    app: &crate::app::App,
    input: RunCodeInput,
) -> Result<Result<RunCodeOutput, crate::domain::DomainError>, anyhow::Error> {
    use crate::util::code_runner::{
        CodeRunnerError, MAX_CODE_LEN, MAX_OPTION_LEN, MAX_STDIN_LEN, RunRequest,
    };

    if input.code.is_empty() {
        return Ok(Err(crate::domain::DomainError::new(
            "code_empty",
            "code must not be empty".to_string(),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }
    if input.code.len() > MAX_CODE_LEN {
        return Ok(Err(crate::domain::DomainError::new(
            "code_too_large",
            format!("code must be at most {MAX_CODE_LEN} bytes"),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }
    if !is_valid_compiler_id(&input.compiler) {
        return Ok(Err(crate::domain::DomainError::new(
            "invalid_compiler",
            "compiler id is invalid".to_string(),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }
    if let Some(stdin) = input.stdin.as_deref()
        && stdin.len() > MAX_STDIN_LEN
    {
        return Ok(Err(crate::domain::DomainError::new(
            "stdin_too_large",
            format!("stdin must be at most {MAX_STDIN_LEN} bytes"),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }
    for opt in [
        input.compiler_options.as_deref(),
        input.compiler_option_raw.as_deref(),
        input.runtime_option_raw.as_deref(),
    ] {
        if let Some(v) = opt
            && v.len() > MAX_OPTION_LEN
        {
            return Ok(Err(crate::domain::DomainError::new(
                "option_too_large",
                format!("each option string must be at most {MAX_OPTION_LEN} bytes"),
                crate::domain::DomainErrorKind::BadInput,
            )));
        }
    }

    let req = RunRequest {
        code: input.code,
        compiler: input.compiler,
        stdin: input.stdin,
        compiler_options: input.compiler_options,
        compiler_option_raw: input.compiler_option_raw,
        runtime_option_raw: input.runtime_option_raw,
    };

    match app.code_runner_client.run(req).await {
        Ok(r) => Ok(Ok(RunCodeOutput {
            status: r.status,
            signal: r.signal,
            compiler_output: r.compiler_output,
            compiler_error: r.compiler_error,
            compiler_message: r.compiler_message,
            program_output: r.program_output,
            program_error: r.program_error,
            program_message: r.program_message,
        })),
        Err(CodeRunnerError::Rejected(msg)) => Ok(Err(crate::domain::DomainError::new(
            "wandbox_rejected",
            msg,
            crate::domain::DomainErrorKind::BadInput,
        ))),
        Err(CodeRunnerError::Upstream(_)) => Ok(Err(crate::domain::DomainError::new(
            "wandbox_unavailable",
            "code runner upstream is unavailable".to_string(),
            crate::domain::DomainErrorKind::Upstream,
        ))),
    }
}

// list compilers

pub struct CompilerSummaryOutput {
    pub name: String,
    pub language: String,
    pub display_name: String,
    pub version: String,
}

pub struct ListCompilersOutput {
    pub compilers: Vec<CompilerSummaryOutput>,
}

pub async fn list_compilers(
    app: &crate::app::App,
) -> Result<Result<ListCompilersOutput, crate::domain::DomainError>, anyhow::Error> {
    use crate::util::code_runner::CodeRunnerError;

    match app.code_runner_client.list_compilers().await {
        Ok(list) => Ok(Ok(ListCompilersOutput {
            compilers: list
                .into_iter()
                .map(|c| CompilerSummaryOutput {
                    name: c.name,
                    language: c.language,
                    display_name: c.display_name,
                    version: c.version,
                })
                .collect(),
        })),
        Err(CodeRunnerError::Rejected(msg)) => Ok(Err(crate::domain::DomainError::new(
            "wandbox_rejected",
            msg,
            crate::domain::DomainErrorKind::BadInput,
        ))),
        Err(CodeRunnerError::Upstream(_)) => Ok(Err(crate::domain::DomainError::new(
            "wandbox_unavailable",
            "code runner upstream is unavailable".to_string(),
            crate::domain::DomainErrorKind::Upstream,
        ))),
    }
}
