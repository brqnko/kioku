// verify access token

pub struct VerifyAccessTokenInput {
    pub access_token: String,
}

pub struct VerifyAccessTokenOutput {
    pub user_id: uuid::Uuid,
    pub jti: uuid::Uuid,
}

pub async fn verify_access_token(
    app: &crate::app::App,
    input: VerifyAccessTokenInput,
) -> Result<Result<VerifyAccessTokenOutput, crate::domain::DomainError>, anyhow::Error> {
    let verified = match app.jwt_service.verify_user_access_token(input.access_token) {
        Ok(ok) => ok,
        Err(_) => {
            return Ok(Err(crate::domain::DomainError::new(
                "invalid_access_token",
                "access token is invalid".to_string(),
                crate::domain::DomainErrorKind::Forbidden,
            )));
        }
    };

    if app
        .jti_blacklist_service
        .is_blacklisted(verified.jti)
        .await?
    {
        return Ok(Err(crate::domain::DomainError::new(
            "revoked_access_token",
            "access token has been revoked".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    Ok(Ok(VerifyAccessTokenOutput {
        user_id: verified.user_id,
        jti: verified.jti,
    }))
}

// oidc start

pub struct OIDCStartInput {}

pub struct OIDCStartOutput {
    pub state_token: String,
    pub redirect_url: String,
    pub nonce: String,
}

pub async fn oidc_start(
    app: &crate::app::App,
    _input: OIDCStartInput,
) -> Result<Result<OIDCStartOutput, crate::domain::DomainError>, anyhow::Error> {
    let code_url = app.oidc_client.code_url().await?;

    Ok(Ok(OIDCStartOutput {
        state_token: code_url.csrf_state,
        redirect_url: code_url.url,
        nonce: code_url.nonce,
    }))
}

// oidc callback

pub struct OIDCCallbackInput {
    pub code: String,
    pub state: String,
    pub expected_state: String,
    pub nonce: String,
    pub display_name: String,
    pub language_code: String,
    pub ip_address: String,
    pub user_agent: String,
}

pub struct OIDCCallbackOutput {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub csrf: String,
    pub refresh_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub redirect_to: String,
}

pub async fn oidc_callback(
    app: &crate::app::App,
    input: OIDCCallbackInput,
) -> Result<Result<OIDCCallbackOutput, crate::domain::DomainError>, anyhow::Error> {
    if input.state != input.expected_state {
        return Ok(Err(crate::domain::DomainError::new(
            "invalid_state",
            "CSRF state mismatch".to_string(),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }

    let result = app
        .oidc_client
        .exchange_and_verify(input.code, input.nonce)
        .await?;

    let mut tx = app.pool.begin().await?;

    let (user, is_new_user) = match app
        .user_repository
        .find_by_iss_sub_for_update(&mut tx, &result.iss, &result.sub)
        .await?
    {
        Ok(Some(ok)) => (ok, false),
        Ok(None) => match super::domain::User::new(
            input.display_name,
            input.language_code,
            result.iss,
            result.sub,
            super::domain::UserOption {
                ..Default::default()
            },
        )? {
            Ok(ok) => (ok, true),
            Err(err) => return Ok(Err(err)),
        },
        Err(err) => return Ok(Err(err)),
    };

    let mut refresh_token = match super::domain::RefreshToken::new(
        user.id,
        input.ip_address,
        input.user_agent,
        super::domain::RefreshTokenOption {
            ..Default::default()
        },
    )? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    let access_token_jti = uuid::Uuid::new_v4();
    let (refresh_token_raw, refresh_token_expires_at) =
        refresh_token.rotate(app.refresh_token_duration, access_token_jti)?;
    let (access_token, access_token_expires_at) = app.jwt_service.sign_user_access_token(
        user.id,
        access_token_jti,
        app.access_token_duration,
    )?;

    if is_new_user {
        match app.user_repository.save(&mut tx, &user).await? {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        };
    }

    match app
        .refresh_token_repository
        .save(&mut tx, &refresh_token)
        .await?
    {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    tx.commit().await?;

    let csrf = crate::util::random::random_string(32);

    Ok(Ok(OIDCCallbackOutput {
        access_token,
        refresh_token: refresh_token_raw,
        access_token_expires_at,
        csrf,
        refresh_token_expires_at,
        redirect_to: format!("{}/dashboard", app.frontend_url),
    }))
}

// logout

pub struct LogoutInput {
    pub refresh_token: String,
}

pub struct LogoutOutput {}

pub async fn logout(
    app: &crate::app::App,
    input: LogoutInput,
) -> Result<Result<LogoutOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let refresh_token = match app
        .refresh_token_repository
        .find_by_token_hash_for_update(&mut tx, &sha256::digest(&input.refresh_token))
        .await?
    {
        Ok(Some(ok)) => ok,
        Ok(None) => {
            return Ok(Err(crate::domain::DomainError::new(
                "invalid_refresh_token",
                "refresh token not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
        Err(err) => return Ok(Err(err)),
    };

    app.jti_blacklist_service
        .add(
            refresh_token.access_token_jti,
            chrono::Utc::now() + app.access_token_duration,
        )
        .await?;

    match app
        .refresh_token_repository
        .remove(&mut tx, refresh_token.id)
        .await?
    {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    tx.commit().await?;

    Ok(Ok(LogoutOutput {}))
}

// refresh

pub struct RefreshInput {
    pub refresh_token: String,
    pub ip_address: String,
    pub user_agent: String,
}

pub struct RefreshOutput {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub refresh_token_expires_at: chrono::DateTime<chrono::Utc>,
}

pub async fn refresh(
    app: &crate::app::App,
    input: RefreshInput,
) -> Result<Result<RefreshOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let mut refresh_token = match app
        .refresh_token_repository
        .find_by_token_hash_for_update(&mut tx, &sha256::digest(&input.refresh_token))
        .await?
    {
        Ok(Some(ok)) => ok,
        Ok(None) => {
            return Ok(Err(crate::domain::DomainError::new(
                "invalid_refresh_token",
                "refresh token not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
        Err(err) => return Ok(Err(err)),
    };

    if chrono::Utc::now() < refresh_token.last_used_at + app.access_token_duration {
        return Ok(Err(crate::domain::DomainError::new(
            "access_token_still_valid",
            "access token has not expired yet".to_string(),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }

    let access_token_jti = uuid::Uuid::new_v4();
    let (refresh_token_raw, refresh_token_expires_at) =
        refresh_token.rotate(app.refresh_token_duration, access_token_jti)?;
    let (access_token, access_token_expires_at) = app.jwt_service.sign_user_access_token(
        refresh_token.user_id,
        access_token_jti,
        app.access_token_duration,
    )?;

    match app
        .refresh_token_repository
        .save(&mut tx, &refresh_token)
        .await?
    {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(RefreshOutput {
        access_token,
        refresh_token: refresh_token_raw,
        access_token_expires_at,
        refresh_token_expires_at,
    }))
}

// get user profile

pub struct GetUserProfileInput {
    pub user_id: uuid::Uuid,
}

pub struct GetUserProfileOutput {
    pub profile: super::query_service::GetUserProfileView,
}

pub async fn get_user_profile(
    app: &crate::app::App,
    input: GetUserProfileInput,
) -> Result<Result<GetUserProfileOutput, crate::domain::DomainError>, anyhow::Error> {
    let profile = match app
        .user_query_service
        .get_user_profile(input.user_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "user_not_found",
                "user not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    Ok(Ok(GetUserProfileOutput { profile }))
}

// update user profile

pub struct UpdateUserProfileInput {
    pub user_id: uuid::Uuid,
    pub display_name: Option<String>,
    pub language_code: Option<String>,
}

pub struct UpdateUserProfileOutput {
    pub user: super::domain::User,
}

pub async fn update_user_profile(
    app: &crate::app::App,
    input: UpdateUserProfileInput,
) -> Result<Result<UpdateUserProfileOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let mut user = match app
        .user_repository
        .find_for_update(&mut tx, input.user_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "user_not_found",
                "user not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if let Some(display_name) = input.display_name {
        match user.set_display_name(display_name) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        }
    }

    if let Some(language_code) = input.language_code {
        match user.set_language_code(language_code) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        }
    }

    match app.user_repository.save(&mut tx, &user).await? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(UpdateUserProfileOutput { user }))
}

// list refresh tokens

pub struct ListRefreshTokensInput {
    pub user_id: uuid::Uuid,
    pub cursor: Option<uuid::Uuid>,
    pub limit: u32,
}

pub struct ListRefreshTokensOutput {
    pub items: Vec<super::query_service::ListRefreshTokensByUserIdView>,
    pub next_cursor: Option<uuid::Uuid>,
}

pub async fn list_refresh_tokens(
    app: &crate::app::App,
    input: ListRefreshTokensInput,
) -> Result<Result<ListRefreshTokensOutput, crate::domain::DomainError>, anyhow::Error> {
    if input.limit > 32 {
        return Ok(Err(crate::domain::DomainError::new(
            "invalid_limit",
            "limit must be 32 or less".to_string(),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }

    let mut rows = app
        .user_query_service
        .list_refresh_tokens_by_user_id(input.user_id, input.cursor, input.limit + 1)
        .await?;

    let next_cursor = if rows.len() as u32 > input.limit {
        rows.pop().map(|r| r.id)
    } else {
        None
    };

    Ok(Ok(ListRefreshTokensOutput {
        items: rows,
        next_cursor,
    }))
}

// revoke refresh token

pub struct RevokeRefreshTokenInput {
    pub user_id: uuid::Uuid,
    pub refresh_token_id: uuid::Uuid,
}

pub struct RevokeRefreshTokenOutput {}

pub async fn revoke_refresh_token(
    app: &crate::app::App,
    input: RevokeRefreshTokenInput,
) -> Result<Result<RevokeRefreshTokenOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let refresh_token = match app
        .refresh_token_repository
        .find_for_update(&mut tx, input.refresh_token_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "refresh_token_not_found",
                "refresh token not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if refresh_token.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "refresh token does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }

    app.jti_blacklist_service
        .add(
            refresh_token.access_token_jti,
            chrono::Utc::now() + app.access_token_duration,
        )
        .await?;

    match app
        .refresh_token_repository
        .remove(&mut tx, refresh_token.id)
        .await?
    {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(RevokeRefreshTokenOutput {}))
}

// remove user

pub struct RemoveUserInput {
    pub user_id: uuid::Uuid,
}

pub struct RemoveUserOutput {}

pub async fn remove_user(
    app: &crate::app::App,
    input: RemoveUserInput,
) -> Result<Result<RemoveUserOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    match app
        .user_repository
        .find_for_update(&mut tx, input.user_id)
        .await?
    {
        Some(_) => {}
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "user_not_found",
                "user not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    }

    match app.user_repository.remove(&mut tx, input.user_id).await? {
        Ok(_) => {}
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(RemoveUserOutput {}))
}

// revoke all refresh tokens

pub struct RevokeAllRefreshTokensInput {
    pub user_id: uuid::Uuid,
}

pub struct RevokeAllRefreshTokensOutput {}

pub async fn revoke_all_refresh_tokens(
    app: &crate::app::App,
    input: RevokeAllRefreshTokensInput,
) -> Result<Result<RevokeAllRefreshTokensOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let tokens = app
        .refresh_token_repository
        .find_all_by_user_id_for_update(&mut tx, input.user_id)
        .await?;

    let expires_at = chrono::Utc::now() + app.access_token_duration;
    for token in &tokens {
        app.jti_blacklist_service
            .add(token.access_token_jti, expires_at)
            .await?;
    }

    match app
        .refresh_token_repository
        .remove_all_by_user_id(&mut tx, input.user_id)
        .await?
    {
        Ok(_) => {}
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(RevokeAllRefreshTokensOutput {}))
}

// get dashboard

pub struct GetDashboardInput {
    pub user_id: uuid::Uuid,
}

pub struct GetDashboardOutput {
    pub dashboard: super::query_service::GetDashboardView,
}

pub async fn get_dashboard(
    app: &crate::app::App,
    input: GetDashboardInput,
) -> Result<Result<GetDashboardOutput, crate::domain::DomainError>, anyhow::Error> {
    let dashboard = match app.user_query_service.get_dashboard(input.user_id).await? {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "user_not_found",
                "user not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    Ok(Ok(GetDashboardOutput { dashboard }))
}

#[cfg(test)]
mod tests {}
