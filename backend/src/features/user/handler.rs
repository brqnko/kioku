fn extract_ip(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn extract_user_agent(headers: &axum::http::HeaderMap) -> String {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}

// oidc start

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct OIDCStartResponse {
    redirect_url: String,
}

#[utoipa::path(
    get,
    path = "/auth/oidc/start",
    responses(
        (status = 200, body = inline(OIDCStartResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn oidc_start(
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
) -> crate::server::HandlerResult<axum::response::Response> {
    use axum::response::IntoResponse as _;

    let input = super::usecase::OIDCStartInput {};
    let output = super::usecase::oidc_start(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            let cookies = [
                format!(
                    "oidc_state={}; HttpOnly; Secure; Path=/; SameSite=Lax; Max-Age=600",
                    o.state_token
                ),
                format!(
                    "oidc_nonce={}; HttpOnly; Secure; Path=/; SameSite=Lax; Max-Age=600",
                    o.nonce
                ),
            ];
            let mut resp = axum::Json(OIDCStartResponse {
                redirect_url: o.redirect_url,
            })
            .into_response();
            for cookie in cookies {
                resp.headers_mut().append(
                    axum::http::header::SET_COOKIE,
                    cookie
                        .parse::<axum::http::HeaderValue>()
                        .expect("invalid cookie value"),
                );
            }
            resp
        })
    }))
}

// oidc callback

#[derive(serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct OIDCCallbackQuery {
    code: String,
    state: String,
}

#[utoipa::path(
    get,
    path = "/auth/oidc/callback",
    params(OIDCCallbackQuery),
    responses(
        (status = 302, description = "Redirect to frontend"),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn oidc_callback(
    jar: axum_extra::extract::CookieJar,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(query): axum::extract::Query<OIDCCallbackQuery>,
) -> crate::server::HandlerResult<axum::response::Response> {
    use axum::response::IntoResponse as _;

    let expected_state = match jar.get("oidc_state").map(|c| c.value().to_string()) {
        Some(v) => v,
        None => {
            return crate::server::schema::HandlerResult(Ok(Err(crate::domain::DomainError::new(
                "missing_state",
                "oidc_state cookie is missing".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            ))));
        }
    };
    let nonce = match jar.get("oidc_nonce").map(|c| c.value().to_string()) {
        Some(v) => v,
        None => {
            return crate::server::schema::HandlerResult(Ok(Err(crate::domain::DomainError::new(
                "missing_nonce",
                "oidc_nonce cookie is missing".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            ))));
        }
    };

    let language_code = headers
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split([',', ';']).next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "en".to_string());

    let input = super::usecase::OIDCCallbackInput {
        code: query.code,
        state: query.state,
        expected_state,
        nonce,
        display_name: "anonymous".to_string(),
        language_code,
        ip_address: extract_ip(&headers),
        user_agent: extract_user_agent(&headers),
    };
    let output = super::usecase::oidc_callback(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| r.map(|o| {
        let access_max_age = (o.access_token_expires_at - chrono::Utc::now()).num_seconds().max(0);
        let refresh_max_age = (o.refresh_token_expires_at - chrono::Utc::now()).num_seconds().max(0);
        let cookies = [
            format!("access_token={}; HttpOnly; Secure; Path=/; SameSite=Strict; Max-Age={}", o.access_token, access_max_age),
            format!("refresh_token={}; HttpOnly; Secure; Path=/; SameSite=Strict; Max-Age={}", o.refresh_token, refresh_max_age),
            format!("csrf={}; Secure; Path=/; SameSite=Strict; Expires=Fri, 31 Dec 9999 23:59:59 GMT", o.csrf),
            "oidc_state=; HttpOnly; Secure; Path=/; SameSite=Lax; Max-Age=0".to_string(),
            "oidc_nonce=; HttpOnly; Secure; Path=/; SameSite=Lax; Max-Age=0".to_string(),
        ];
        let mut resp = axum::response::Redirect::to(&o.redirect_to).into_response();
        for cookie in cookies {
            resp.headers_mut().append(axum::http::header::SET_COOKIE, cookie.parse::<axum::http::HeaderValue>().expect("invalid cookie value"));
        }
        resp
    })))
}

// logout

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 204, description = "Logged out"),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn logout(
    jar: axum_extra::extract::CookieJar,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
) -> crate::server::HandlerResult<axum::response::Response> {
    use axum::response::IntoResponse as _;

    let refresh_token = match jar.get("refresh_token").map(|c| c.value().to_string()) {
        Some(v) => v,
        None => {
            return crate::server::schema::HandlerResult(Ok(Err(crate::domain::DomainError::new(
                "missing_refresh_token",
                "refresh_token cookie is missing".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            ))));
        }
    };

    let input = super::usecase::LogoutInput { refresh_token };
    let output = super::usecase::logout(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|_| {
            let clear = [
                "access_token=; HttpOnly; Secure; Path=/; SameSite=Strict; Max-Age=0",
                "refresh_token=; HttpOnly; Secure; Path=/; SameSite=Strict; Max-Age=0",
                "csrf=; Path=/; SameSite=Strict; Max-Age=0",
            ];
            let mut resp = axum::http::StatusCode::NO_CONTENT.into_response();
            for cookie in clear {
                resp.headers_mut().append(
                    axum::http::header::SET_COOKIE,
                    cookie
                        .parse::<axum::http::HeaderValue>()
                        .expect("invalid cookie value"),
                );
            }
            resp
        })
    }))
}

// refresh

#[utoipa::path(
    post,
    path = "/auth/refresh",
    responses(
        (status = 204, description = "Refreshed"),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn refresh(
    jar: axum_extra::extract::CookieJar,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    headers: axum::http::HeaderMap,
) -> crate::server::HandlerResult<axum::response::Response> {
    use axum::response::IntoResponse as _;

    let refresh_token = match jar.get("refresh_token").map(|c| c.value().to_string()) {
        Some(v) => v,
        None => {
            return crate::server::schema::HandlerResult(Ok(Err(crate::domain::DomainError::new(
                "missing_refresh_token",
                "refresh_token cookie is missing".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            ))));
        }
    };

    let input = super::usecase::RefreshInput {
        refresh_token,
        ip_address: extract_ip(&headers),
        user_agent: extract_user_agent(&headers),
    };
    let output = super::usecase::refresh(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            let access_max_age = (o.access_token_expires_at - chrono::Utc::now())
                .num_seconds()
                .max(0);
            let refresh_max_age = (o.refresh_token_expires_at - chrono::Utc::now())
                .num_seconds()
                .max(0);
            let cookies = [
                format!(
                    "access_token={}; HttpOnly; Secure; Path=/; SameSite=Strict; Max-Age={}",
                    o.access_token, access_max_age
                ),
                format!(
                    "refresh_token={}; HttpOnly; Secure; Path=/; SameSite=Strict; Max-Age={}",
                    o.refresh_token, refresh_max_age
                ),
            ];
            let mut resp = axum::http::StatusCode::NO_CONTENT.into_response();
            for cookie in cookies {
                resp.headers_mut().append(
                    axum::http::header::SET_COOKIE,
                    cookie
                        .parse::<axum::http::HeaderValue>()
                        .expect("invalid cookie value"),
                );
            }
            resp
        })
    }))
}

// get user profile

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct UserProfileResponse {
    id: String,
    display_name: String,
    language_code: String,
    joined_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    get,
    path = "/users/me",
    security(("cookieAuth" = [], "csrfToken" = [])),
    responses(
        (status = 200, body = inline(UserProfileResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn get_user_profile(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
) -> crate::server::HandlerResult<axum::Json<UserProfileResponse>> {
    let input = super::usecase::GetUserProfileInput { user_id };
    let output = super::usecase::get_user_profile(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(UserProfileResponse {
                id: o.profile.id.to_string(),
                display_name: o.profile.display_name,
                language_code: o.profile.language_code,
                joined_at: o.profile.joined_at,
            })
        })
    }))
}

// update user profile

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateUserProfileBody {
    #[schema(max_length = 32)]
    display_name: Option<String>,
    #[schema(max_length = 7)]
    language_code: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/users/me",
    security(("cookieAuth" = [], "csrfToken" = [])),
    request_body = inline(UpdateUserProfileBody),
    responses(
        (status = 200, body = inline(UserProfileResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn update_user_profile(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::Json(body): axum::Json<UpdateUserProfileBody>,
) -> crate::server::HandlerResult<axum::Json<UserProfileResponse>> {
    let input = super::usecase::UpdateUserProfileInput {
        user_id,
        display_name: body.display_name,
        language_code: body.language_code,
    };
    let output = super::usecase::update_user_profile(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(UserProfileResponse {
                id: o.user.id.to_string(),
                display_name: o.user.display_name.0,
                language_code: o.user.language_code.0,
                joined_at: o.user.joined_at,
            })
        })
    }))
}

// remove user

#[utoipa::path(
    delete,
    path = "/users/me",
    security(("cookieAuth" = [], "csrfToken" = [])),
    responses(
        (status = 204, description = "Deleted"),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn remove_user(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
) -> crate::server::HandlerResult<axum::http::StatusCode> {
    let input = super::usecase::RemoveUserInput { user_id };
    let output = super::usecase::remove_user(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|_| axum::http::StatusCode::NO_CONTENT)),
    )
}

// list sessions

#[derive(serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListSessionsQuery {
    pub cursor: Option<uuid::Uuid>,
    #[param(minimum = 1, maximum = 32)]
    pub limit: u32,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SessionItem {
    id: String,
    ip_address: String,
    user_agent: String,
    activated_at: chrono::DateTime<chrono::Utc>,
    last_used_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ListSessionsResponse {
    #[schema(inline)]
    items: Vec<SessionItem>,
    next_cursor: Option<String>,
}

#[utoipa::path(
    get,
    path = "/users/me/sessions",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(ListSessionsQuery),
    responses(
        (status = 200, body = inline(ListSessionsResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn list_sessions(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Query(query): axum::extract::Query<ListSessionsQuery>,
) -> crate::server::HandlerResult<axum::Json<ListSessionsResponse>> {
    let input = super::usecase::ListRefreshTokensInput {
        user_id,
        cursor: query.cursor,
        limit: query.limit,
    };
    let output = super::usecase::list_refresh_tokens(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(ListSessionsResponse {
                items: o
                    .items
                    .into_iter()
                    .map(|item| SessionItem {
                        id: item.id.to_string(),
                        ip_address: item.ip_address,
                        user_agent: item.user_agent,
                        activated_at: item.activated_at,
                        last_used_at: item.last_used_at,
                        expires_at: item.expires_at,
                    })
                    .collect::<Vec<SessionItem>>(),
                next_cursor: o.next_cursor.map(|c| c.to_string()),
            })
        })
    }))
}

// revoke session

#[utoipa::path(
    delete,
    path = "/users/me/sessions/{session_id}",
    security(("cookieAuth" = [], "csrfToken" = [])),
    params(
        ("session_id" = uuid::Uuid, Path, description = "Session ID to revoke"),
    ),
    responses(
        (status = 204, description = "Revoked"),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn revoke_session(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
    axum::extract::Path(session_id): axum::extract::Path<uuid::Uuid>,
) -> crate::server::HandlerResult<axum::http::StatusCode> {
    let input = super::usecase::RevokeRefreshTokenInput {
        user_id,
        refresh_token_id: session_id,
    };
    let output = super::usecase::revoke_refresh_token(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|_| axum::http::StatusCode::NO_CONTENT)),
    )
}

// revoke all sessions

#[utoipa::path(
    delete,
    path = "/users/me/sessions",
    security(("cookieAuth" = [], "csrfToken" = [])),
    responses(
        (status = 204, description = "All sessions revoked"),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn revoke_all_sessions(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
) -> crate::server::HandlerResult<axum::http::StatusCode> {
    let input = super::usecase::RevokeAllRefreshTokensInput { user_id };
    let output = super::usecase::revoke_all_refresh_tokens(&app, input).await;

    crate::server::schema::HandlerResult(
        output.map(|r| r.map(|_| axum::http::StatusCode::NO_CONTENT)),
    )
}

// get dashboard

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct DashboardRecentFileItem {
    id: String,
    name: String,
    user_id: String,
    storage_type: u8,
    storage_id: String,
    changed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct DashboardResponse {
    ai_learning_summary: String,
    ai_learning_summary_updated_at: chrono::DateTime<chrono::Utc>,
    #[schema(inline)]
    recent_seen_files: Vec<DashboardRecentFileItem>,
}

#[utoipa::path(
    get,
    path = "/users/me/dashboard",
    security(("cookieAuth" = [], "csrfToken" = [])),
    responses(
        (status = 200, body = inline(DashboardResponse)),
        (status = 400, body = crate::server::schema::ErrorBody),
        (status = 401, body = crate::server::schema::ErrorBody),
        (status = 403, body = crate::server::schema::ErrorBody),
        (status = 404, body = crate::server::schema::ErrorBody),
        (status = 500, body = crate::server::schema::ErrorBody),
    )
)]
pub async fn get_dashboard(
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    axum::extract::State(app): axum::extract::State<std::sync::Arc<crate::app::App>>,
) -> crate::server::HandlerResult<axum::Json<DashboardResponse>> {
    let input = super::usecase::GetDashboardInput { user_id };
    let output = super::usecase::get_dashboard(&app, input).await;

    crate::server::schema::HandlerResult(output.map(|r| {
        r.map(|o| {
            axum::Json(DashboardResponse {
                ai_learning_summary: o.dashboard.ai_learning_summary,
                ai_learning_summary_updated_at: o.dashboard.ai_learning_summary_updated_at,
                recent_seen_files: o
                    .dashboard
                    .recent_seen_files
                    .into_iter()
                    .map(|f| DashboardRecentFileItem {
                        id: f.id.to_string(),
                        name: f.name,
                        user_id: f.user_id.to_string(),
                        storage_type: f.storage_type,
                        storage_id: f.storage_id.to_string(),
                        changed_at: f.changed_at,
                    })
                    .collect::<Vec<DashboardRecentFileItem>>(),
            })
        })
    }))
}

pub fn public_router() -> utoipa_axum::router::OpenApiRouter<std::sync::Arc<crate::app::App>> {
    use utoipa_axum::routes;

    utoipa_axum::router::OpenApiRouter::new()
        .routes(routes!(oidc_start))
        .routes(routes!(oidc_callback))
        .routes(routes!(logout))
        .routes(routes!(refresh))
}

pub fn protected_router() -> utoipa_axum::router::OpenApiRouter<std::sync::Arc<crate::app::App>> {
    use utoipa_axum::routes;

    utoipa_axum::router::OpenApiRouter::new()
        .routes(routes!(get_user_profile, update_user_profile, remove_user))
        .routes(routes!(list_sessions, revoke_all_sessions))
        .routes(routes!(revoke_session))
        .routes(routes!(get_dashboard))
}
