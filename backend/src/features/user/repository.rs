#[async_trait::async_trait]
pub trait RefreshTokenRepository<C>: Send + Sync {
    async fn find_for_update(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::RefreshToken>, anyhow::Error>;
    async fn find_all_by_user_id_for_update(
        &self,
        c: &mut C,
        user_id: uuid::Uuid,
    ) -> Result<Vec<super::domain::RefreshToken>, anyhow::Error>;
    async fn find_by_token_hash_for_update(
        &self,
        c: &mut C,
        token_hash: &str,
    ) -> Result<
        Result<Option<super::domain::RefreshToken>, crate::domain::DomainError>,
        anyhow::Error,
    >;
    async fn save(
        &self,
        c: &mut C,
        refresh_token: &super::domain::RefreshToken,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove_all_by_user_id(
        &self,
        c: &mut C,
        user_id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct RefreshTokenRepositoryImpl {}

impl Default for RefreshTokenRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl RefreshTokenRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl RefreshTokenRepository<sqlx::MySqlConnection> for RefreshTokenRepositoryImpl {
    async fn find_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::RefreshToken>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT refresh_token_id, user_id, token_hash, generation, ip_address, user_agent,
                   access_token_jti, activated_at, last_used_at, expires_at
            FROM refresh_token
            WHERE refresh_token_id = ?
            FOR UPDATE
            "#,
            id.as_bytes().as_slice(),
        )
        .fetch_optional(c)
        .await?;

        match row {
            Some(r) => Ok(Some(super::domain::RefreshToken {
                id: uuid::Uuid::from_slice(&r.refresh_token_id)?,
                user_id: uuid::Uuid::from_slice(&r.user_id)?,
                token_hash: r.token_hash,
                generation: r.generation,
                ip_address: r.ip_address,
                user_agent: r.user_agent,
                access_token_jti: uuid::Uuid::from_slice(&r.access_token_jti)?,
                activated_at: r.activated_at.and_utc(),
                last_used_at: r.last_used_at.and_utc(),
                expires_at: r.expires_at.and_utc(),
            })),
            None => Ok(None),
        }
    }

    async fn find_by_token_hash_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        token_hash: &str,
    ) -> Result<
        Result<Option<super::domain::RefreshToken>, crate::domain::DomainError>,
        anyhow::Error,
    > {
        let row = sqlx::query!(
            r#"
            SELECT refresh_token_id, user_id, token_hash, generation, ip_address, user_agent,
                   access_token_jti, activated_at, last_used_at, expires_at
            FROM refresh_token
            WHERE token_hash = ?
            FOR UPDATE
            "#,
            token_hash,
        )
        .fetch_optional(c)
        .await?;

        match row {
            Some(r) => Ok(Ok(Some(super::domain::RefreshToken {
                id: uuid::Uuid::from_slice(&r.refresh_token_id)?,
                user_id: uuid::Uuid::from_slice(&r.user_id)?,
                token_hash: r.token_hash,
                generation: r.generation,
                ip_address: r.ip_address,
                user_agent: r.user_agent,
                access_token_jti: uuid::Uuid::from_slice(&r.access_token_jti)?,
                activated_at: r.activated_at.and_utc(),
                last_used_at: r.last_used_at.and_utc(),
                expires_at: r.expires_at.and_utc(),
            }))),
            None => Ok(Ok(None)),
        }
    }

    async fn save(
        &self,
        c: &mut sqlx::MySqlConnection,
        refresh_token: &super::domain::RefreshToken,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        sqlx::query!(
            r#"
            INSERT INTO refresh_token
                (refresh_token_id, user_id, token_hash, generation, ip_address, user_agent,
                 access_token_jti, activated_at, last_used_at, expires_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                token_hash       = VALUES(token_hash),
                generation       = VALUES(generation),
                ip_address       = VALUES(ip_address),
                user_agent       = VALUES(user_agent),
                access_token_jti = VALUES(access_token_jti),
                last_used_at     = VALUES(last_used_at),
                expires_at       = VALUES(expires_at)
            "#,
            refresh_token.id.as_bytes().as_slice(),
            refresh_token.user_id.as_bytes().as_slice(),
            refresh_token.token_hash,
            refresh_token.generation,
            refresh_token.ip_address,
            refresh_token.user_agent,
            refresh_token.access_token_jti.as_bytes().as_slice(),
            refresh_token.activated_at,
            refresh_token.last_used_at,
            refresh_token.expires_at,
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }

    async fn find_all_by_user_id_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        user_id: uuid::Uuid,
    ) -> Result<Vec<super::domain::RefreshToken>, anyhow::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT refresh_token_id, user_id, token_hash, generation, ip_address, user_agent,
                   access_token_jti, activated_at, last_used_at, expires_at
            FROM refresh_token
            WHERE user_id = ?
            FOR UPDATE
            "#,
            user_id.as_bytes().as_slice(),
        )
        .fetch_all(c)
        .await?;

        rows.into_iter()
            .map(|r| {
                Ok(super::domain::RefreshToken {
                    id: uuid::Uuid::from_slice(&r.refresh_token_id)?,
                    user_id: uuid::Uuid::from_slice(&r.user_id)?,
                    token_hash: r.token_hash,
                    generation: r.generation,
                    ip_address: r.ip_address,
                    user_agent: r.user_agent,
                    access_token_jti: uuid::Uuid::from_slice(&r.access_token_jti)?,
                    activated_at: r.activated_at.and_utc(),
                    last_used_at: r.last_used_at.and_utc(),
                    expires_at: r.expires_at.and_utc(),
                })
            })
            .collect::<Result<Vec<super::domain::RefreshToken>, anyhow::Error>>()
    }

    async fn remove(
        &self,
        c: &mut sqlx::MySqlConnection,
        id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        sqlx::query!(
            "DELETE FROM refresh_token WHERE refresh_token_id = ?",
            id.as_bytes().as_slice(),
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }

    async fn remove_all_by_user_id(
        &self,
        c: &mut sqlx::MySqlConnection,
        user_id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        sqlx::query!(
            "DELETE FROM refresh_token WHERE user_id = ?",
            user_id.as_bytes().as_slice(),
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }
}

#[async_trait::async_trait]
pub trait UserRepository<C>: Send + Sync {
    async fn find_for_update(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::User>, anyhow::Error>;
    async fn find_by_iss_sub_for_update(
        &self,
        c: &mut C,
        iss: &str,
        sub: &str,
    ) -> Result<Result<Option<super::domain::User>, crate::domain::DomainError>, anyhow::Error>;
    async fn save(
        &self,
        c: &mut C,
        user: &super::domain::User,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
    async fn remove(
        &self,
        c: &mut C,
        id: uuid::Uuid,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error>;
}

pub struct UserRepositoryImpl {}

impl Default for UserRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl UserRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl UserRepository<sqlx::MySqlConnection> for UserRepositoryImpl {
    async fn find_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        id: uuid::Uuid,
    ) -> Result<Option<super::domain::User>, anyhow::Error> {
        let row = sqlx::query!(
            r#"
            SELECT
                user_id,
                display_name,
                language_code,
                joined_at,
                iss,
                sub,
                recent_seen_file_ids as "recent_seen_file_ids: sqlx::types::Json<Vec<uuid::Uuid>>",
                ai_learning_summary,
                ai_learning_summary_updated_at
            FROM user
            WHERE user_id = ?
            FOR UPDATE
            "#,
            id.as_bytes().as_slice(),
        )
        .fetch_optional(c)
        .await?;

        match row {
            Some(r) => Ok(Some(super::domain::User {
                id: uuid::Uuid::from_slice(&r.user_id)?,
                display_name: super::domain::DisplayName(r.display_name),
                language_code: super::domain::LanguageCode(r.language_code),
                joined_at: r.joined_at.and_utc(),
                iss: super::domain::Iss(r.iss),
                sub: super::domain::Sub(r.sub),
                recent_seen_file_ids: r.recent_seen_file_ids.0,
                ai_learning_summary: r.ai_learning_summary,
                ai_learning_summary_updated_at: r.ai_learning_summary_updated_at.and_utc(),
            })),
            None => Ok(None),
        }
    }

    async fn find_by_iss_sub_for_update(
        &self,
        c: &mut sqlx::MySqlConnection,
        iss: &str,
        sub: &str,
    ) -> Result<Result<Option<super::domain::User>, crate::domain::DomainError>, anyhow::Error>
    {
        let row = sqlx::query!(
            r#"
            SELECT
                user_id,
                display_name,
                language_code,
                joined_at,
                iss,
                sub,
                recent_seen_file_ids as "recent_seen_file_ids: sqlx::types::Json<Vec<uuid::Uuid>>",
                ai_learning_summary,
                ai_learning_summary_updated_at
            FROM user
            WHERE iss = ? AND sub = ?
            FOR UPDATE
            "#,
            iss,
            sub,
        )
        .fetch_optional(c)
        .await?;

        match row {
            Some(r) => Ok(Ok(Some(super::domain::User {
                id: uuid::Uuid::from_slice(&r.user_id)?,
                display_name: super::domain::DisplayName(r.display_name),
                language_code: super::domain::LanguageCode(r.language_code),
                joined_at: r.joined_at.and_utc(),
                iss: super::domain::Iss(r.iss),
                sub: super::domain::Sub(r.sub),
                recent_seen_file_ids: r.recent_seen_file_ids.0,
                ai_learning_summary: r.ai_learning_summary,
                ai_learning_summary_updated_at: r.ai_learning_summary_updated_at.and_utc(),
            }))),
            None => Ok(Ok(None)),
        }
    }

    async fn save(
        &self,
        c: &mut sqlx::MySqlConnection,
        user: &super::domain::User,
    ) -> Result<Result<(), crate::domain::DomainError>, anyhow::Error> {
        sqlx::query!(
            r#"
            INSERT INTO user
                (user_id, display_name, language_code, joined_at, iss, sub,
                 recent_seen_file_ids, ai_learning_summary, ai_learning_summary_updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                display_name                   = VALUES(display_name),
                language_code                  = VALUES(language_code),
                recent_seen_file_ids           = VALUES(recent_seen_file_ids),
                ai_learning_summary            = VALUES(ai_learning_summary),
                ai_learning_summary_updated_at = VALUES(ai_learning_summary_updated_at)
            "#,
            user.id.as_bytes().as_slice(),
            user.display_name.0,
            user.language_code.0,
            user.joined_at,
            user.iss.0,
            user.sub.0,
            sqlx::types::Json(&user.recent_seen_file_ids) as _,
            user.ai_learning_summary,
            user.ai_learning_summary_updated_at,
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
            "DELETE FROM user WHERE user_id = ?",
            id.as_bytes().as_slice(),
        )
        .execute(c)
        .await?;

        Ok(Ok(()))
    }
}
