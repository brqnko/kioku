// create project

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateProjectBody {
    #[schema(max_length = 256)]
    name: String,
    #[schema(max_length = 512)]
    description: String,
}

#[utoipa::path(
    post,
    path = "/projects",
    security(("Bearer" = [])),
    request_body = inline(CreateProjectBody),
    responses(
        (status = 200, body = inline(ProjectResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn create_project(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::Json(body): axum::Json<CreateProjectBody>,
) -> crate::server::HandlerResult<axum::Json<ProjectResponse>> {
    let input = super::usecase::CreateProjectInput {
        user_id,
        name: body.name,
        description: body.description,
    };
    let output = super::usecase::create_project(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(ProjectResponse {
                id: o.project.id.to_string(),
                created_by: o.project.created_by.to_string(),
                name: o.project.name.0,
                description: o.project.description.0,
                indexed_at: o.project.indexed_at,
                last_seen_at: o.project.last_seen_at,
                last_seen_file_id: o.project.last_seen_file_id.to_string(),
            })
        })
    }))
}

// list projects

#[derive(serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ListProjectsOrder {
    LastSeenAtAsc,
    LastSeenAtDesc,
}

impl From<ListProjectsOrder> for super::query_service::ListProjectsByUserIdOrder {
    fn from(value: ListProjectsOrder) -> Self {
        match value {
            ListProjectsOrder::LastSeenAtAsc => Self::LastSeenAtAsc,
            ListProjectsOrder::LastSeenAtDesc => Self::LastSeenAtDesc,
        }
    }
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListProjectsQuery {
    #[param(inline)]
    pub order: ListProjectsOrder,
    pub cursor_last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cursor_project_id: Option<uuid::Uuid>,
    #[param(minimum = 1, maximum = 32)]
    pub limit: u32,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ProjectItem {
    id: String,
    created_by: String,
    name: String,
    description: String,
    indexed_at: chrono::DateTime<chrono::Utc>,
    last_seen_at: chrono::DateTime<chrono::Utc>,
    last_seen_file_id: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListProjectsCursor {
    last_seen_at: chrono::DateTime<chrono::Utc>,
    project_id: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListProjectsResponse {
    #[schema(inline)]
    items: Vec<ProjectItem>,
    #[schema(inline)]
    next_cursor: Option<ListProjectsCursor>,
}

#[utoipa::path(
    get,
    path = "/projects",
    security(("Bearer" = [])),
    params(ListProjectsQuery),
    responses(
        (status = 200, body = inline(ListProjectsResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn list_projects(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Query(query): axum::extract::Query<ListProjectsQuery>,
) -> crate::server::HandlerResult<axum::Json<ListProjectsResponse>> {
    let cursor = match (query.cursor_last_seen_at, query.cursor_project_id) {
        (Some(last_seen_at), Some(project_id)) => {
            Some(super::query_service::ListProjectsByUserIdCursor {
                last_seen_at,
                project_id,
            })
        }
        (None, None) => None,
        _ => {
            return crate::server::schema::HandlerResult(Ok(Err(crate::domain::DomainError::new(
                "invalid_cursor",
                "cursor_last_seen_at and cursor_project_id must be provided together".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            ))));
        }
    };

    let input = super::usecase::ListProjectsInput {
        user_id,
        order: query.order.into(),
        cursor,
        limit: query.limit,
    };
    let output = super::usecase::list_projects(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(ListProjectsResponse {
                items: o
                    .items
                    .into_iter()
                    .map(|item| ProjectItem {
                        id: item.id.to_string(),
                        created_by: item.created_by.to_string(),
                        name: item.name,
                        description: item.description,
                        indexed_at: item.indexed_at,
                        last_seen_at: item.last_seen_at,
                        last_seen_file_id: item.last_seen_file_id.to_string(),
                    })
                    .collect(),
                next_cursor: o.next_cursor.map(|c| ListProjectsCursor {
                    last_seen_at: c.last_seen_at,
                    project_id: c.project_id.to_string(),
                }),
            })
        })
    }))
}

// get project

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ProjectResponse {
    id: String,
    created_by: String,
    name: String,
    description: String,
    indexed_at: chrono::DateTime<chrono::Utc>,
    last_seen_at: chrono::DateTime<chrono::Utc>,
    last_seen_file_id: String,
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}",
    security(("Bearer" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, body = inline(ProjectResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn get_project(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
) -> crate::server::HandlerResult<axum::Json<ProjectResponse>> {
    let input = super::usecase::GetProjectInput {
        user_id,
        project_id,
    };
    let output = super::usecase::get_project(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(ProjectResponse {
                id: o.project.id.to_string(),
                created_by: o.project.created_by.to_string(),
                name: o.project.name.0,
                description: o.project.description.0,
                indexed_at: o.project.indexed_at,
                last_seen_at: o.project.last_seen_at,
                last_seen_file_id: o.project.last_seen_file_id.to_string(),
            })
        })
    }))
}

// update project

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateProjectBody {
    #[schema(max_length = 256)]
    name: Option<String>,
    #[schema(max_length = 512)]
    description: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/projects/{project_id}",
    security(("Bearer" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
    ),
    request_body = inline(UpdateProjectBody),
    responses(
        (status = 200, body = inline(ProjectResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn update_project(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
    axum::Json(body): axum::Json<UpdateProjectBody>,
) -> crate::server::HandlerResult<axum::Json<ProjectResponse>> {
    let input = super::usecase::UpdateProjectInput {
        user_id,
        project_id,
        name: body.name,
        description: body.description,
    };
    let output = super::usecase::update_project(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(ProjectResponse {
                id: o.project.id.to_string(),
                created_by: o.project.created_by.to_string(),
                name: o.project.name.0,
                description: o.project.description.0,
                indexed_at: o.project.indexed_at,
                last_seen_at: o.project.last_seen_at,
                last_seen_file_id: o.project.last_seen_file_id.to_string(),
            })
        })
    }))
}

// remove project

#[utoipa::path(
    delete,
    path = "/projects/{project_id}",
    security(("Bearer" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
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
pub async fn remove_project(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
) -> crate::server::HandlerResult<axum::http::StatusCode> {
    let input = super::usecase::RemoveProjectInput {
        user_id,
        project_id,
    };
    let output = super::usecase::remove_project(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|_| axum::http::StatusCode::NO_CONTENT)),
    )
}

pub fn protected_router() -> utoipa_axum::router::OpenApiRouter<std::sync::Arc<crate::app::App>> {
    use utoipa_axum::routes;

    utoipa_axum::router::OpenApiRouter::new()
        .routes(routes!(create_project, list_projects))
        .routes(routes!(get_project, update_project, remove_project))
}
