// shared schemas

#[derive(serde::Serialize, utoipa::ToSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ChatMessageRole {
    User,
    Assistant,
}

impl From<super::domain::ChatMessageRole> for ChatMessageRole {
    fn from(value: super::domain::ChatMessageRole) -> Self {
        match value {
            super::domain::ChatMessageRole::User => Self::User,
            super::domain::ChatMessageRole::Assistant => Self::Assistant,
        }
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ChatMessageResponse {
    #[schema(inline)]
    role: ChatMessageRole,
    content: String,
    sent_at: chrono::DateTime<chrono::Utc>,
}

impl From<super::domain::ChatMessage> for ChatMessageResponse {
    fn from(value: super::domain::ChatMessage) -> Self {
        Self {
            role: value.role.into(),
            content: value.content,
            sent_at: value.sent_at,
        }
    }
}

// create chat

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateChatBody {
    #[schema(max_length = 256)]
    name: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ChatResponse {
    id: String,
    name: String,
    user_id: String,
    project_id: String,
    started_at: chrono::DateTime<chrono::Utc>,
    last_activity_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/chats",
    security(("Bearer" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
    ),
    request_body = inline(CreateChatBody),
    responses(
        (status = 200, body = inline(ChatResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn create_chat(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
    axum::Json(body): axum::Json<CreateChatBody>,
) -> crate::server::HandlerResult<axum::Json<ChatResponse>> {
    let input = super::usecase::CreateChatInput {
        user_id,
        project_id,
        name: body.name,
    };
    let output = super::usecase::create_chat(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(ChatResponse {
                id: o.chat.id.to_string(),
                name: o.chat.name.0,
                user_id: o.chat.user_id.to_string(),
                project_id: o.chat.project_id.to_string(),
                started_at: o.chat.started_at,
                last_activity_at: o.chat.last_activity_at,
            })
        })
    }))
}

// list chats

#[derive(serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListChatsQuery {
    pub cursor_last_activity_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cursor_chat_id: Option<uuid::Uuid>,
    #[param(minimum = 1, maximum = 32)]
    pub limit: u32,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ChatListItemResponse {
    id: String,
    name: String,
    user_id: String,
    project_id: String,
    started_at: chrono::DateTime<chrono::Utc>,
    last_activity_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListChatsCursorResponse {
    last_activity_at: chrono::DateTime<chrono::Utc>,
    chat_id: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListChatsResponse {
    #[schema(inline)]
    items: Vec<ChatListItemResponse>,
    #[schema(inline)]
    next_cursor: Option<ListChatsCursorResponse>,
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/chats",
    security(("Bearer" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
        ListChatsQuery,
    ),
    responses(
        (status = 200, body = inline(ListChatsResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn list_chats(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(query): axum::extract::Query<ListChatsQuery>,
) -> crate::server::HandlerResult<axum::Json<ListChatsResponse>> {
    let cursor = match (query.cursor_last_activity_at, query.cursor_chat_id) {
        (Some(last_activity_at), Some(chat_id)) => Some(super::usecase::ListChatsCursor {
            last_activity_at,
            chat_id,
        }),
        (None, None) => None,
        _ => {
            return crate::server::schema::HandlerResult(Ok(Err(crate::domain::DomainError::new(
                "invalid_cursor",
                "cursor_last_activity_at and cursor_chat_id must be provided together".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            ))));
        }
    };

    let input = super::usecase::ListChatsInput {
        user_id,
        project_id,
        cursor,
        limit: query.limit,
    };
    let output = super::usecase::list_chats(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            let items = o
                .items
                .into_iter()
                .map(|item| ChatListItemResponse {
                    id: item.id.to_string(),
                    name: item.name,
                    user_id: item.user_id.to_string(),
                    project_id: item.project_id.to_string(),
                    started_at: item.started_at,
                    last_activity_at: item.last_activity_at,
                })
                .collect::<Vec<ChatListItemResponse>>();
            let next_cursor = o.next_cursor.map(|c| ListChatsCursorResponse {
                last_activity_at: c.last_activity_at,
                chat_id: c.chat_id.to_string(),
            });
            axum::Json(ListChatsResponse { items, next_cursor })
        })
    }))
}

// get chat (with full message history)

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct GetChatResponse {
    id: String,
    name: String,
    user_id: String,
    project_id: String,
    #[schema(inline)]
    messages: Vec<ChatMessageResponse>,
    started_at: chrono::DateTime<chrono::Utc>,
    last_activity_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/chats/{chat_id}",
    security(("Bearer" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
        ("chat_id" = uuid::Uuid, Path, description = "Chat ID"),
    ),
    responses(
        (status = 200, body = inline(GetChatResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn get_chat(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path((project_id, chat_id)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> crate::server::HandlerResult<axum::Json<GetChatResponse>> {
    let input = super::usecase::GetChatInput {
        user_id,
        project_id,
        chat_id,
    };
    let output = super::usecase::get_chat(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(GetChatResponse {
                id: o.chat.id.to_string(),
                name: o.chat.name,
                user_id: o.chat.user_id.to_string(),
                project_id: o.chat.project_id.to_string(),
                messages: o
                    .chat
                    .messages
                    .into_iter()
                    .map(ChatMessageResponse::from)
                    .collect::<Vec<ChatMessageResponse>>(),
                started_at: o.chat.started_at,
                last_activity_at: o.chat.last_activity_at,
            })
        })
    }))
}

// send message

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct SendMessageBody {
    #[schema(max_length = 8192)]
    content: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SendMessageResponse {
    #[schema(inline)]
    user_message: ChatMessageResponse,
    #[schema(inline)]
    assistant_message: ChatMessageResponse,
    last_activity_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/chats/{chat_id}/messages",
    security(("Bearer" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
        ("chat_id" = uuid::Uuid, Path, description = "Chat ID"),
    ),
    request_body = inline(SendMessageBody),
    responses(
        (status = 200, body = inline(SendMessageResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn send_message(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path((project_id, chat_id)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
    axum::Json(body): axum::Json<SendMessageBody>,
) -> crate::server::HandlerResult<axum::Json<SendMessageResponse>> {
    let input = super::usecase::SendMessageInput {
        user_id,
        project_id,
        chat_id,
        content: body.content,
    };
    let output = super::usecase::send_message(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(SendMessageResponse {
                user_message: o.user_message.into(),
                assistant_message: o.assistant_message.into(),
                last_activity_at: o.last_activity_at,
            })
        })
    }))
}

// update chat

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateChatBody {
    #[schema(max_length = 256, nullable)]
    name: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/projects/{project_id}/chats/{chat_id}",
    security(("Bearer" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
        ("chat_id" = uuid::Uuid, Path, description = "Chat ID"),
    ),
    request_body = inline(UpdateChatBody),
    responses(
        (status = 200, body = inline(ChatResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn update_chat(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path((project_id, chat_id)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
    axum::Json(body): axum::Json<UpdateChatBody>,
) -> crate::server::HandlerResult<axum::Json<ChatResponse>> {
    let input = super::usecase::UpdateChatInput {
        user_id,
        project_id,
        chat_id,
        name: body.name,
    };
    let output = super::usecase::update_chat(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(ChatResponse {
                id: o.chat.id.to_string(),
                name: o.chat.name.0,
                user_id: o.chat.user_id.to_string(),
                project_id: o.chat.project_id.to_string(),
                started_at: o.chat.started_at,
                last_activity_at: o.chat.last_activity_at,
            })
        })
    }))
}

// remove chat

#[utoipa::path(
    delete,
    path = "/projects/{project_id}/chats/{chat_id}",
    security(("Bearer" = [])),
    params(
        ("project_id" = uuid::Uuid, Path, description = "Project ID"),
        ("chat_id" = uuid::Uuid, Path, description = "Chat ID"),
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
pub async fn remove_chat(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path((project_id, chat_id)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> crate::server::HandlerResult<axum::http::StatusCode> {
    let input = super::usecase::RemoveChatInput {
        user_id,
        project_id,
        chat_id,
    };
    let output = super::usecase::remove_chat(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|_| axum::http::StatusCode::NO_CONTENT)),
    )
}

pub fn protected_router() -> utoipa_axum::router::OpenApiRouter<std::sync::Arc<crate::app::App>> {
    use utoipa_axum::routes;

    utoipa_axum::router::OpenApiRouter::new()
        .routes(routes!(create_chat, list_chats))
        .routes(routes!(get_chat, update_chat, remove_chat))
        .routes(routes!(send_message))
}
