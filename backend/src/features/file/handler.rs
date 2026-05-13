#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParentKind {
    Project,
    Folder,
}

impl ParentKind {
    fn as_u8(self) -> u8 {
        match self {
            Self::Project => 0,
            Self::Folder => 1,
        }
    }

    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Project),
            1 => Some(Self::Folder),
            _ => None,
        }
    }
}

// request upload url

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct RequestUploadUrlBody {
    content_type: String,
    #[schema(minimum = 1, maximum = 16777216)]
    content_length: i64,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct RequestUploadUrlResponse {
    storage_id: String,
    url: String,
    method: String,
    content_type: String,
    content_length: i64,
    expires_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    post,
    path = "/files/upload-url",
    security(("cookieAuth" = [], "csrfToken" = [])),
    request_body = inline(RequestUploadUrlBody),
    responses(
        (status = 200, body = inline(RequestUploadUrlResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn request_upload_url(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::Json(body): axum::Json<RequestUploadUrlBody>,
) -> crate::server::HandlerResult<axum::Json<RequestUploadUrlResponse>> {
    let input = super::usecase::RequestUploadUrlInput {
        user_id,
        content_type: body.content_type,
        content_length: body.content_length,
    };
    let output = super::usecase::request_upload_url(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(RequestUploadUrlResponse {
                storage_id: o.storage_id.to_string(),
                url: o.url,
                method: o.method,
                content_type: o.content_type,
                content_length: o.content_length,
                expires_at: o.expires_at,
            })
        })
    }))
}

// create file

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateFileBody {
    #[schema(max_length = 256)]
    name: String,
    #[schema(max_length = 1024)]
    description: String,
    storage_id: Option<uuid::Uuid>,
    #[schema(max_length = 16777216)]
    text: Option<String>,
    parent_id: uuid::Uuid,
    #[schema(inline)]
    parent_kind: ParentKind,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct FileResponse {
    id: String,
    name: String,
    description: String,
    user_id: String,
    storage_type: u8,
    storage_id: String,
    file_size: u64,
    parent_id: String,
    #[schema(inline)]
    parent_kind: ParentKind,
    uploaded_at: chrono::DateTime<chrono::Utc>,
    changed_at: chrono::DateTime<chrono::Utc>,
}

fn file_response(file: super::domain::File) -> FileResponse {
    let parent_kind = ParentKind::from_u8(file.parent.kind()).expect("invalid parent kind");
    FileResponse {
        id: file.id.to_string(),
        name: file.name.0,
        description: file.description.0,
        user_id: file.user_id.to_string(),
        storage_type: file.storage_type as u8,
        storage_id: file.storage_id.to_string(),
        file_size: file.file_size,
        parent_id: file.parent.id().to_string(),
        parent_kind,
        uploaded_at: file.uploaded_at,
        changed_at: file.changed_at,
    }
}

#[utoipa::path(
    post,
    path = "/files",
    security(("cookieAuth" = [], "csrfToken" = [])),
    request_body = inline(CreateFileBody),
    responses(
        (status = 200, body = inline(FileResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn create_file(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::Json(body): axum::Json<CreateFileBody>,
) -> crate::server::HandlerResult<axum::Json<FileResponse>> {
    let input = super::usecase::CreateFileInput {
        user_id,
        name: body.name,
        description: body.description,
        storage_id: body.storage_id,
        text: body.text,
        parent_id: body.parent_id,
        parent_kind: body.parent_kind.as_u8(),
    };
    let output = super::usecase::create_file(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|o| axum::Json(file_response(o.file)))),
    )
}

// get file content

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FileContentBody {
    Url {
        url: String,
        method: String,
        expires_at: chrono::DateTime<chrono::Utc>,
    },
    Text {
        content: String,
    },
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct GetFileContentResponse {
    #[schema(inline)]
    file: FileResponse,
    #[schema(inline)]
    content: FileContentBody,
}

#[utoipa::path(
    get,
    path = "/files/{file_id}/content",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("file_id" = uuid::Uuid, Path, description = "File ID"),
    ),
    responses(
        (status = 200, body = inline(GetFileContentResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn get_file_content(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(file_id): axum::extract::Path<uuid::Uuid>,
) -> crate::server::HandlerResult<axum::Json<GetFileContentResponse>> {
    let input = super::usecase::GetFileContentInput { user_id, file_id };
    let output = super::usecase::get_file_content(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            let parent_kind = ParentKind::from_u8(o.file.parent_kind).expect("invalid parent kind");
            let file = FileResponse {
                id: o.file.id.to_string(),
                name: o.file.name,
                description: o.file.description,
                user_id: o.file.user_id.to_string(),
                storage_type: o.file.storage_type,
                storage_id: o.file.storage_id.to_string(),
                file_size: o.file.file_size,
                parent_id: o.file.parent_id.to_string(),
                parent_kind,
                uploaded_at: o.file.uploaded_at,
                changed_at: o.file.changed_at,
            };
            let content = match o.content {
                super::usecase::FileContent::Url {
                    url,
                    method,
                    expires_at,
                } => FileContentBody::Url {
                    url,
                    method,
                    expires_at,
                },
                super::usecase::FileContent::Text { content } => FileContentBody::Text { content },
            };
            axum::Json(GetFileContentResponse { file, content })
        })
    }))
}

// get file raw

#[utoipa::path(
    get,
    path = "/files/{file_id}/raw",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("file_id" = uuid::Uuid, Path, description = "File ID"),
    ),
    responses(
        (status = 302, description = "Redirect to presigned URL"),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn get_file_raw(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(file_id): axum::extract::Path<uuid::Uuid>,
) -> crate::server::HandlerResult<axum::response::Redirect> {
    let input = super::usecase::GetFileRawUrlInput { user_id, file_id };
    let output = super::usecase::get_file_raw_url(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|o| axum::response::Redirect::temporary(&o.url))),
    )
}

// update file

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateFileBody {
    #[schema(max_length = 256)]
    name: Option<String>,
    #[schema(max_length = 1024)]
    description: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/files/{file_id}",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("file_id" = uuid::Uuid, Path, description = "File ID"),
    ),
    request_body = inline(UpdateFileBody),
    responses(
        (status = 200, body = inline(FileResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn update_file(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(file_id): axum::extract::Path<uuid::Uuid>,
    axum::Json(body): axum::Json<UpdateFileBody>,
) -> crate::server::HandlerResult<axum::Json<FileResponse>> {
    let input = super::usecase::UpdateFileInput {
        user_id,
        file_id,
        name: body.name,
        description: body.description,
    };
    let output = super::usecase::update_file(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|o| axum::Json(file_response(o.file)))),
    )
}

// update file text

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateFileTextBody {
    #[schema(max_length = 16777216)]
    text: String,
}

#[utoipa::path(
    put,
    path = "/files/{file_id}/text",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("file_id" = uuid::Uuid, Path, description = "File ID"),
    ),
    request_body = inline(UpdateFileTextBody),
    responses(
        (status = 200, body = inline(FileResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn update_file_text(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(file_id): axum::extract::Path<uuid::Uuid>,
    axum::Json(body): axum::Json<UpdateFileTextBody>,
) -> crate::server::HandlerResult<axum::Json<FileResponse>> {
    let input = super::usecase::UpdateFileTextInput {
        user_id,
        file_id,
        text: body.text,
    };
    let output = super::usecase::update_file_text(&app, input).await;

    // if let Ok(Ok(o)) = &output {
    //     let app = app.clone();
    //     let file_id = o.file.id;
    //     tokio::spawn(async move {
    //         let input = super::usecase::IndexFileInput { file_id };
    //         match super::usecase::index_file(&app, input).await {
    //             Ok(Ok(_)) => tracing::info!(%file_id, "index_file finished"),
    //             Ok(Err(err)) => tracing::error!(%file_id, ?err, "index_file domain error"),
    //             Err(err) => tracing::error!(%file_id, ?err, "index_file failed"),
    //         }
    //     });
    // }

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|o| axum::Json(file_response(o.file)))),
    )
}

// remove file

#[utoipa::path(
    delete,
    path = "/files/{file_id}",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("file_id" = uuid::Uuid, Path, description = "File ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn remove_file(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(file_id): axum::extract::Path<uuid::Uuid>,
) -> crate::server::HandlerResult<axum::http::StatusCode> {
    let input = super::usecase::RemoveFileInput { user_id, file_id };
    let output = super::usecase::remove_file(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|_| axum::http::StatusCode::NO_CONTENT)),
    )
}

// create folder

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateFolderBody {
    parent_id: uuid::Uuid,
    #[schema(inline)]
    parent_kind: ParentKind,
    #[schema(max_length = 256)]
    name: String,
    #[schema(max_length = 1024)]
    description: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct FolderResponse {
    id: String,
    parent_id: String,
    #[schema(inline)]
    parent_kind: ParentKind,
    depth: u8,
    name: String,
    description: String,
    user_id: String,
    uploaded_at: chrono::DateTime<chrono::Utc>,
    changed_at: chrono::DateTime<chrono::Utc>,
}

fn folder_response(folder: super::domain::Folder) -> FolderResponse {
    let parent_kind = ParentKind::from_u8(folder.parent.kind()).expect("invalid parent kind");
    FolderResponse {
        id: folder.id.to_string(),
        parent_id: folder.parent.id().to_string(),
        parent_kind,
        depth: folder.depth,
        name: folder.name.0,
        description: folder.description.0,
        user_id: folder.user_id.to_string(),
        uploaded_at: folder.uploaded_at,
        changed_at: folder.changed_at,
    }
}

#[utoipa::path(
    post,
    path = "/folders",
    security(("cookieAuth" = [], "csrfToken" = [])),
    request_body = inline(CreateFolderBody),
    responses(
        (status = 200, body = inline(FolderResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn create_folder(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::Json(body): axum::Json<CreateFolderBody>,
) -> crate::server::HandlerResult<axum::Json<FolderResponse>> {
    let input = super::usecase::CreateFolderInput {
        user_id,
        parent_id: body.parent_id,
        parent_kind: body.parent_kind.as_u8(),
        name: body.name,
        description: body.description,
    };
    let output = super::usecase::create_folder(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|o| axum::Json(folder_response(o.folder)))),
    )
}

// get folder

#[utoipa::path(
    get,
    path = "/folders/{folder_id}",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("folder_id" = uuid::Uuid, Path, description = "Folder ID"),
    ),
    responses(
        (status = 200, body = inline(FolderResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn get_folder(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(folder_id): axum::extract::Path<uuid::Uuid>,
) -> crate::server::HandlerResult<axum::Json<FolderResponse>> {
    let input = super::usecase::GetFolderInput { user_id, folder_id };
    let output = super::usecase::get_folder(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            let parent_kind =
                ParentKind::from_u8(o.folder.parent_kind).expect("invalid parent kind");
            axum::Json(FolderResponse {
                id: o.folder.id.to_string(),
                parent_id: o.folder.parent_id.to_string(),
                parent_kind,
                depth: o.folder.depth,
                name: o.folder.name,
                description: o.folder.description,
                user_id: o.folder.user_id.to_string(),
                uploaded_at: o.folder.uploaded_at,
                changed_at: o.folder.changed_at,
            })
        })
    }))
}

// update folder

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateFolderBody {
    #[schema(max_length = 256)]
    name: Option<String>,
    #[schema(max_length = 1024)]
    description: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/folders/{folder_id}",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("folder_id" = uuid::Uuid, Path, description = "Folder ID"),
    ),
    request_body = inline(UpdateFolderBody),
    responses(
        (status = 200, body = inline(FolderResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn update_folder(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(folder_id): axum::extract::Path<uuid::Uuid>,
    axum::Json(body): axum::Json<UpdateFolderBody>,
) -> crate::server::HandlerResult<axum::Json<FolderResponse>> {
    let input = super::usecase::UpdateFolderInput {
        user_id,
        folder_id,
        name: body.name,
        description: body.description,
    };
    let output = super::usecase::update_folder(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|o| axum::Json(folder_response(o.folder)))),
    )
}

// remove folder

#[utoipa::path(
    delete,
    path = "/folders/{folder_id}",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("folder_id" = uuid::Uuid, Path, description = "Folder ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn remove_folder(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(folder_id): axum::extract::Path<uuid::Uuid>,
) -> crate::server::HandlerResult<axum::http::StatusCode> {
    let input = super::usecase::RemoveFolderInput { user_id, folder_id };
    let output = super::usecase::remove_folder(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|_| axum::http::StatusCode::NO_CONTENT)),
    )
}

// list ancestors

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AncestorItem {
    Project(#[schema(inline)] AncestorEntry),
    Folder(#[schema(inline)] AncestorEntry),
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AncestorEntry {
    id: String,
    name: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListAncestorsResponse {
    #[schema(inline)]
    ancestors: Vec<AncestorItem>,
}

fn build_list_ancestors_response(
    ancestors: Vec<super::query_service::AncestorView>,
) -> ListAncestorsResponse {
    let items = ancestors
        .into_iter()
        .map(|a| {
            let entry = AncestorEntry {
                id: a.id.to_string(),
                name: a.name,
            };
            match a.kind {
                0 => AncestorItem::Project(entry),
                _ => AncestorItem::Folder(entry),
            }
        })
        .collect::<Vec<AncestorItem>>();
    ListAncestorsResponse { ancestors: items }
}

#[utoipa::path(
    get,
    path = "/folders/{folder_id}/ancestors",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("folder_id" = uuid::Uuid, Path, description = "Folder ID"),
    ),
    responses(
        (status = 200, body = inline(ListAncestorsResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn get_folder_ancestors(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(folder_id): axum::extract::Path<uuid::Uuid>,
) -> crate::server::HandlerResult<axum::Json<ListAncestorsResponse>> {
    let input = super::usecase::GetFolderAncestorsInput { user_id, folder_id };
    let output = super::usecase::get_folder_ancestors(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|o| axum::Json(build_list_ancestors_response(o.ancestors)))),
    )
}

#[utoipa::path(
    get,
    path = "/files/{file_id}/ancestors",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("file_id" = uuid::Uuid, Path, description = "File ID"),
    ),
    responses(
        (status = 200, body = inline(ListAncestorsResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn get_file_ancestors(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(file_id): axum::extract::Path<uuid::Uuid>,
) -> crate::server::HandlerResult<axum::Json<ListAncestorsResponse>> {
    let input = super::usecase::GetFileAncestorsInput { user_id, file_id };
    let output = super::usecase::get_file_ancestors(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|o| axum::Json(build_list_ancestors_response(o.ancestors)))),
    )
}

// list children

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ListChildrenCursorPhase {
    Folders,
    Files,
}

impl From<ListChildrenCursorPhase> for super::query_service::ListChildrenByParentPhase {
    fn from(value: ListChildrenCursorPhase) -> Self {
        match value {
            ListChildrenCursorPhase::Folders => Self::Folders,
            ListChildrenCursorPhase::Files => Self::Files,
        }
    }
}

impl From<super::query_service::ListChildrenByParentPhase> for ListChildrenCursorPhase {
    fn from(value: super::query_service::ListChildrenByParentPhase) -> Self {
        match value {
            super::query_service::ListChildrenByParentPhase::Folders => Self::Folders,
            super::query_service::ListChildrenByParentPhase::Files => Self::Files,
        }
    }
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListChildrenQuery {
    #[param(inline)]
    pub cursor_phase: Option<ListChildrenCursorPhase>,
    pub cursor_name: Option<String>,
    pub cursor_id: Option<uuid::Uuid>,
    #[param(minimum = 1, maximum = 32)]
    pub limit: u32,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListChildrenFolderItem {
    id: String,
    parent_id: String,
    #[schema(inline)]
    parent_kind: ParentKind,
    name: String,
    description: String,
    user_id: String,
    uploaded_at: chrono::DateTime<chrono::Utc>,
    changed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListChildrenFileItem {
    id: String,
    name: String,
    description: String,
    user_id: String,
    storage_id: String,
    file_size: u64,
    parent_id: String,
    #[schema(inline)]
    parent_kind: ParentKind,
    uploaded_at: chrono::DateTime<chrono::Utc>,
    changed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ListChildrenItem {
    Folder(#[schema(inline)] ListChildrenFolderItem),
    File(#[schema(inline)] ListChildrenFileItem),
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListChildrenCursor {
    #[schema(inline)]
    phase: ListChildrenCursorPhase,
    name: String,
    id: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListChildrenResponse {
    #[schema(inline)]
    items: Vec<ListChildrenItem>,
    #[schema(inline)]
    next_cursor: Option<ListChildrenCursor>,
}

fn build_list_children_cursor(
    query: &ListChildrenQuery,
) -> Result<Option<super::query_service::ListChildrenByParentCursor>, crate::domain::DomainError> {
    match (query.cursor_phase, &query.cursor_name, query.cursor_id) {
        (Some(phase), Some(name), Some(id)) => {
            Ok(Some(super::query_service::ListChildrenByParentCursor {
                phase: phase.into(),
                name: name.clone(),
                id,
            }))
        }
        (None, None, None) => Ok(None),
        _ => Err(crate::domain::DomainError::new(
            "invalid_cursor",
            "cursor_phase, cursor_name, cursor_id must be provided together".to_string(),
            crate::domain::DomainErrorKind::BadInput,
        )),
    }
}

fn build_list_children_response(
    output: super::usecase::ListChildrenOutput,
) -> ListChildrenResponse {
    let items = output
        .items
        .into_iter()
        .map(|item| match item {
            super::query_service::ListChildrenByParentView::Folder(f) => {
                let parent_kind = ParentKind::from_u8(f.parent_kind).expect("invalid parent kind");
                ListChildrenItem::Folder(ListChildrenFolderItem {
                    id: f.id.to_string(),
                    parent_id: f.parent_id.to_string(),
                    parent_kind,
                    name: f.name,
                    description: f.description,
                    user_id: f.user_id.to_string(),
                    uploaded_at: f.uploaded_at,
                    changed_at: f.changed_at,
                })
            }
            super::query_service::ListChildrenByParentView::File(f) => {
                let parent_kind = ParentKind::from_u8(f.parent_kind).expect("invalid parent kind");
                ListChildrenItem::File(ListChildrenFileItem {
                    id: f.id.to_string(),
                    name: f.name,
                    description: f.description,
                    user_id: f.user_id.to_string(),
                    storage_id: f.storage_id.to_string(),
                    file_size: f.file_size,
                    parent_id: f.parent_id.to_string(),
                    parent_kind,
                    uploaded_at: f.uploaded_at,
                    changed_at: f.changed_at,
                })
            }
        })
        .collect::<Vec<ListChildrenItem>>();
    let next_cursor = output.next_cursor.map(|c| ListChildrenCursor {
        phase: c.phase.into(),
        name: c.name,
        id: c.id.to_string(),
    });
    ListChildrenResponse { items, next_cursor }
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/children",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
        ListChildrenQuery,
    ),
    responses(
        (status = 200, body = inline(ListChildrenResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn list_project_children(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(query): axum::extract::Query<ListChildrenQuery>,
) -> crate::server::HandlerResult<axum::Json<ListChildrenResponse>> {
    let cursor = match build_list_children_cursor(&query) {
        Ok(ok) => ok,
        Err(err) => {
            return crate::server::schema::HandlerResult(Ok(Err(err)));
        }
    };

    let input = super::usecase::ListChildrenInput {
        user_id,
        parent_id: project_id,
        parent_kind: ParentKind::Project.as_u8(),
        cursor,
        limit: query.limit,
    };
    let output = super::usecase::list_children(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|o| axum::Json(build_list_children_response(o)))),
    )
}

#[utoipa::path(
    get,
    path = "/folders/{folder_id}/children",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("folder_id" = uuid::Uuid, Path, description = "Folder ID"),
        ListChildrenQuery,
    ),
    responses(
        (status = 200, body = inline(ListChildrenResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn list_folder_children(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(folder_id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(query): axum::extract::Query<ListChildrenQuery>,
) -> crate::server::HandlerResult<axum::Json<ListChildrenResponse>> {
    let cursor = match build_list_children_cursor(&query) {
        Ok(ok) => ok,
        Err(err) => {
            return crate::server::schema::HandlerResult(Ok(Err(err)));
        }
    };

    let input = super::usecase::ListChildrenInput {
        user_id,
        parent_id: folder_id,
        parent_kind: ParentKind::Folder.as_u8(),
        cursor,
        limit: query.limit,
    };
    let output = super::usecase::list_children(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|o| axum::Json(build_list_children_response(o)))),
    )
}

// run code

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct RunCodeBody {
    #[schema(max_length = 65536)]
    code: String,
    #[schema(max_length = 64)]
    compiler: String,
    #[schema(max_length = 32768)]
    stdin: Option<String>,
    #[schema(max_length = 4096)]
    compiler_options: Option<String>,
    #[schema(max_length = 4096)]
    compiler_option_raw: Option<String>,
    #[schema(max_length = 4096)]
    runtime_option_raw: Option<String>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct RunCodeResponse {
    status: Option<String>,
    signal: Option<String>,
    compiler_output: Option<String>,
    compiler_error: Option<String>,
    compiler_message: Option<String>,
    program_output: Option<String>,
    program_error: Option<String>,
    program_message: Option<String>,
}

#[utoipa::path(
    post,
    path = "/files/run",
    security(("cookieAuth" = [], "csrfToken" = [])),
    request_body = inline(RunCodeBody),
    responses(
        (status = 200, body = inline(RunCodeResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
        (status = 502, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn run_code(
    axum::extract::Extension(_user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::Json(body): axum::Json<RunCodeBody>,
) -> crate::server::HandlerResult<axum::Json<RunCodeResponse>> {
    let input = super::usecase::RunCodeInput {
        code: body.code,
        compiler: body.compiler,
        stdin: body.stdin,
        compiler_options: body.compiler_options,
        compiler_option_raw: body.compiler_option_raw,
        runtime_option_raw: body.runtime_option_raw,
    };
    let output = super::usecase::run_code(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(RunCodeResponse {
                status: o.status,
                signal: o.signal,
                compiler_output: o.compiler_output,
                compiler_error: o.compiler_error,
                compiler_message: o.compiler_message,
                program_output: o.program_output,
                program_error: o.program_error,
                program_message: o.program_message,
            })
        })
    }))
}

// list compilers

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct CompilerSummary {
    name: String,
    language: String,
    display_name: String,
    version: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListCompilersResponse {
    #[schema(inline)]
    compilers: Vec<CompilerSummary>,
}

#[utoipa::path(
    get,
    path = "/code/compilers",
    security(("cookieAuth" = [], "csrfToken" = [])),
    responses(
        (status = 200, body = inline(ListCompilersResponse)),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
        (status = 502, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn list_compilers(
    axum::extract::Extension(_user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
) -> crate::server::HandlerResult<axum::Json<ListCompilersResponse>> {
    let output = super::usecase::list_compilers(&app).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(ListCompilersResponse {
                compilers: o
                    .compilers
                    .into_iter()
                    .map(|c| CompilerSummary {
                        name: c.name,
                        language: c.language,
                        display_name: c.display_name,
                        version: c.version,
                    })
                    .collect(),
            })
        })
    }))
}

pub fn protected_router() -> utoipa_axum::router::OpenApiRouter<std::sync::Arc<crate::app::App>> {
    use utoipa_axum::routes;

    utoipa_axum::router::OpenApiRouter::new()
        .routes(routes!(request_upload_url))
        .routes(routes!(create_file))
        .routes(routes!(get_file_content))
        .routes(routes!(get_file_raw))
        .routes(routes!(update_file_text))
        .routes(routes!(update_file, remove_file))
        .routes(routes!(create_folder))
        .routes(routes!(get_folder, update_folder, remove_folder))
        .routes(routes!(get_folder_ancestors))
        .routes(routes!(get_file_ancestors))
        .routes(routes!(list_project_children))
        .routes(routes!(list_folder_children))
        .routes(routes!(run_code))
        .routes(routes!(list_compilers))
}
