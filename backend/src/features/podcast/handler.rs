// create podcast

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreatePodcastBody {
    #[schema(max_length = 256)]
    name: String,
    #[schema(max_length = 1024)]
    description: String,
    used_file_ids: Vec<uuid::Uuid>,
    /// One of: F1, F2, F3, F4, F5, M1, M2, M3, M4, M5
    voice_style: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct CreatePodcastResponse {
    podcast_id: String,
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/podcasts",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
    ),
    request_body = inline(CreatePodcastBody),
    responses(
        (status = 200, body = inline(CreatePodcastResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn create_podcast(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
    axum::Json(body): axum::Json<CreatePodcastBody>,
) -> crate::server::HandlerResult<axum::Json<CreatePodcastResponse>> {
    let input = super::usecase::CreatePodcastInput {
        user_id,
        project_id,
        name: body.name,
        description: body.description,
        used_file_ids: body.used_file_ids,
        voice_style: body.voice_style,
    };
    let output = super::usecase::create_podcast(&app, input).await;

    if let Ok(Ok(o)) = &output {
        let app = app.clone();
        let podcast_id = o.podcast_id;
        tokio::spawn(async move {
            let input = super::usecase::GeneratePodcastInput { podcast_id };
            match super::usecase::generate_podcast(&app, input).await {
                Ok(Ok(_)) => tracing::info!(%podcast_id, "generate_podcast finished"),
                Ok(Err(err)) => {
                    tracing::error!(%podcast_id, ?err, "generate_podcast domain error")
                }
                Err(err) => tracing::error!(%podcast_id, ?err, "generate_podcast failed"),
            }
        });
    }

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(CreatePodcastResponse {
                podcast_id: o.podcast_id.to_string(),
            })
        })
    }))
}

// get podcast

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct PodcastScriptEntryResponse {
    speaker: String,
    text: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct GetPodcastResponse {
    id: String,
    name: String,
    description: String,
    user_id: String,
    project_id: String,
    used_file_ids: Vec<String>,
    #[schema(inline)]
    podcast_script: Vec<PodcastScriptEntryResponse>,
    audio_url: String,
    podcast_created_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/podcasts/{podcast_id}",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
        ("podcast_id" = uuid::Uuid, Path, description = "Podcast ID"),
    ),
    responses(
        (status = 200, body = inline(GetPodcastResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn get_podcast(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path((project_id, podcast_id)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> crate::server::HandlerResult<axum::Json<GetPodcastResponse>> {
    let input = super::usecase::GetPodcastInput {
        user_id,
        project_id,
        podcast_id,
    };
    let output = super::usecase::get_podcast(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(GetPodcastResponse {
                id: o.view.id.to_string(),
                name: o.view.name,
                description: o.view.description,
                user_id: o.view.user_id.to_string(),
                project_id: o.view.project_id.to_string(),
                used_file_ids: o
                    .view
                    .used_file_ids
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>(),
                podcast_script: o
                    .view
                    .podcast_script
                    .into_iter()
                    .map(|e| PodcastScriptEntryResponse {
                        speaker: e.speaker,
                        text: e.text,
                    })
                    .collect::<Vec<PodcastScriptEntryResponse>>(),
                audio_url: o.audio_url,
                podcast_created_at: o.view.podcast_created_at,
            })
        })
    }))
}

// list podcasts

#[derive(serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListPodcastsQuery {
    pub cursor_created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cursor_podcast_id: Option<uuid::Uuid>,
    #[param(minimum = 1, maximum = 32)]
    pub limit: u32,
}

#[derive(serde::Serialize, utoipa::ToSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum PodcastStatus {
    Generating,
    Generated,
}

impl From<super::usecase::PodcastStatus> for PodcastStatus {
    fn from(value: super::usecase::PodcastStatus) -> Self {
        match value {
            super::usecase::PodcastStatus::Generating => Self::Generating,
            super::usecase::PodcastStatus::Generated => Self::Generated,
        }
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct PodcastListItemResponse {
    id: String,
    name: String,
    description: String,
    user_id: String,
    project_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    #[schema(inline)]
    status: PodcastStatus,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListPodcastsCursorResponse {
    created_at: chrono::DateTime<chrono::Utc>,
    podcast_id: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListPodcastsResponse {
    #[schema(inline)]
    items: Vec<PodcastListItemResponse>,
    #[schema(inline)]
    next_cursor: Option<ListPodcastsCursorResponse>,
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/podcasts",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
        ListPodcastsQuery,
    ),
    responses(
        (status = 200, body = inline(ListPodcastsResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn list_podcasts(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(query): axum::extract::Query<ListPodcastsQuery>,
) -> crate::server::HandlerResult<axum::Json<ListPodcastsResponse>> {
    let cursor = match (query.cursor_created_at, query.cursor_podcast_id) {
        (Some(created_at), Some(podcast_id)) => Some(super::usecase::ListPodcastsCursor {
            created_at,
            podcast_id,
        }),
        (None, None) => None,
        _ => {
            return crate::server::schema::HandlerResult(Ok(Err(crate::domain::DomainError::new(
                "invalid_cursor",
                "cursor_created_at and cursor_podcast_id must be provided together".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            ))));
        }
    };

    let input = super::usecase::ListPodcastsInput {
        user_id,
        project_id,
        cursor,
        limit: query.limit,
    };
    let output = super::usecase::list_podcasts(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            let items = o
                .items
                .into_iter()
                .map(|item| PodcastListItemResponse {
                    id: item.id.to_string(),
                    name: item.name,
                    description: item.description,
                    user_id: item.user_id.to_string(),
                    project_id: item.project_id.to_string(),
                    created_at: item.created_at,
                    status: item.status.into(),
                })
                .collect::<Vec<PodcastListItemResponse>>();
            let next_cursor = o.next_cursor.map(|c| ListPodcastsCursorResponse {
                created_at: c.created_at,
                podcast_id: c.podcast_id.to_string(),
            });
            axum::Json(ListPodcastsResponse { items, next_cursor })
        })
    }))
}

// remove podcast

#[utoipa::path(
    delete,
    path = "/projects/{project_id}/podcasts/{podcast_id}",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
        ("podcast_id" = uuid::Uuid, Path, description = "Podcast ID"),
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
pub async fn remove_podcast(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path((project_id, podcast_id)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> crate::server::HandlerResult<axum::http::StatusCode> {
    let input = super::usecase::RemovePodcastInput {
        user_id,
        project_id,
        podcast_id,
    };
    let output = super::usecase::remove_podcast(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|_| axum::http::StatusCode::NO_CONTENT)),
    )
}

// update podcast

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdatePodcastBody {
    #[schema(max_length = 256)]
    name: Option<String>,
    #[schema(max_length = 1024)]
    description: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/projects/{project_id}/podcasts/{podcast_id}",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
        ("podcast_id" = uuid::Uuid, Path, description = "Podcast ID"),
    ),
    request_body = inline(UpdatePodcastBody),
    responses(
        (status = 200, body = inline(GetPodcastResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn update_podcast(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path((project_id, podcast_id)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
    axum::Json(body): axum::Json<UpdatePodcastBody>,
) -> crate::server::HandlerResult<axum::Json<GetPodcastResponse>> {
    let input = super::usecase::UpdatePodcastInput {
        user_id,
        project_id,
        podcast_id,
        name: body.name,
        description: body.description,
    };
    let output = super::usecase::update_podcast(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(GetPodcastResponse {
                id: o.view.id.to_string(),
                name: o.view.name,
                description: o.view.description,
                user_id: o.view.user_id.to_string(),
                project_id: o.view.project_id.to_string(),
                used_file_ids: o
                    .view
                    .used_file_ids
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>(),
                podcast_script: o
                    .view
                    .podcast_script
                    .into_iter()
                    .map(|e| PodcastScriptEntryResponse {
                        speaker: e.speaker,
                        text: e.text,
                    })
                    .collect::<Vec<PodcastScriptEntryResponse>>(),
                audio_url: o.audio_url,
                podcast_created_at: o.view.podcast_created_at,
            })
        })
    }))
}

pub fn protected_router() -> utoipa_axum::router::OpenApiRouter<std::sync::Arc<crate::app::App>> {
    use utoipa_axum::routes;

    utoipa_axum::router::OpenApiRouter::new()
        .routes(routes!(create_podcast, list_podcasts))
        .routes(routes!(get_podcast, update_podcast, remove_podcast))
}
