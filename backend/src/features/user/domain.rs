use std::ops::Add as _;

pub struct DisplayName(pub String);

impl DisplayName {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 32 {
            return Err(crate::domain::DomainError::new(
                "invalid_display_name",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ))
        }

        Ok(Self(string))
    }
}

pub struct LanguageCode(String);

impl LanguageCode {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 7 {
            return Err(crate::domain::DomainError::new(
                "invalid_language_code",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ))
        }

        Ok(Self(string))
    }
}

pub struct User {
    pub id: uuid::Uuid,
    pub display_name: DisplayName,
    pub language_code: LanguageCode,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub iss: String,
    pub sub: String,
    pub recent_seen_file_ids: Vec<uuid::Uuid>,
    pub ai_learning_summary: String,
    pub ai_learning_summary_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Default)]
pub struct UserOption {
    pub id: Option<uuid::Uuid>,
    pub joined_at: Option<chrono::DateTime<chrono::Utc>>,
    pub recent_seen_file_ids: Option<Vec<uuid::Uuid>>,
    pub ai_learning_summary: Option<String>,
    pub ai_learning_summary_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl User {
    pub fn new(
        display_name: String,
        language_code: String,
        iss: String,
        sub: String,
        option: UserOption,
    ) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
        let display_name = match DisplayName::new(display_name) {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };
        let language_code = match LanguageCode::new(language_code) {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };

        if let Some(recent_file_ids) = &option.recent_seen_file_ids && recent_file_ids.len() > 4 {
            return Err(anyhow::anyhow!("recent_file_ids len is too long: {}", recent_file_ids.len()));
        }

        if let Some(ai_learning_summary) = &option.ai_learning_summary && ai_learning_summary.len() > 512 {
            return Err(anyhow::anyhow!("ai learning summary len is too long: {}", ai_learning_summary.len()));
        }

        Ok(Ok(Self {
            id: option.id.unwrap_or(uuid::Uuid::new_v4()),
            display_name,
            language_code,
            joined_at: option.joined_at.unwrap_or(chrono::Utc::now()),
            iss,
            sub,
            recent_seen_file_ids: option.recent_seen_file_ids.unwrap_or(Vec::new()),
            ai_learning_summary: option.ai_learning_summary.unwrap_or("".to_owned()),
            ai_learning_summary_updated_at: option.ai_learning_summary_updated_at.unwrap_or(chrono::Utc::now()),
        }))
    }

    pub fn set_display_name(&mut self, display_name: String) -> Result<(), crate::domain::DomainError> {
        self.display_name = DisplayName::new(display_name)?;
        Ok(())
    }

    pub fn set_language_code(&mut self, language_code: String) -> Result<(), crate::domain::DomainError> {
        self.language_code = LanguageCode::new(language_code)?;
        Ok(())
    }

    pub fn push_recent_seen_file_id(&mut self, file_id: uuid::Uuid) -> Result<(), crate::domain::DomainError> {
        // TODO

        Ok(())
    }

    pub fn set_ai_learning_summary(&mut self, summary: String) -> Result<(), anyhow::Error> {
        let len = summary.len();
        if len > 512 {
            return Err(anyhow::anyhow!("ai learning summary len is too long: {len}"))
        }

        self.ai_learning_summary = summary;

        Ok(())
    }
}

pub struct RefreshToken {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub token_hash: String,
    pub generation: i32,
    pub ip_address: String,
    pub user_agent: String,
    pub access_token_jti: uuid::Uuid,
    pub activated_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Default)]
pub struct RefreshTokenOption {
    pub id: Option<uuid::Uuid>,
    pub token_hash: Option<String>,
    pub generation: Option<i32>,
    pub access_token_jti: Option<uuid::Uuid>,
    pub activated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl RefreshToken {
    pub fn new(user_id: uuid::Uuid, ip_address: String, user_agent: String, option: RefreshTokenOption) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
        Ok(Ok(Self {
            id: option.id.unwrap_or(uuid::Uuid::new_v4()),
            user_id: user_id,
            token_hash: option.token_hash.unwrap_or("".to_owned()),
            generation: option.generation.unwrap_or(1),
            ip_address: ip_address,
            user_agent: user_agent,
            access_token_jti: option.access_token_jti.unwrap_or(uuid::Uuid::nil()),
            activated_at: option.activated_at.unwrap_or(chrono::Utc::now()),
            last_used_at: option.last_used_at.unwrap_or(chrono::Utc::now()),
            expires_at: option.expires_at.unwrap_or(chrono::Utc::now()),
        }))
    }

    pub fn rotate(&mut self, refresh_token_duration: chrono::Duration, access_token_jti: uuid::Uuid) -> Result<(String, chrono::DateTime<chrono::Utc>), anyhow::Error> {
        let new_refresh_token = crate::util::random::random_string(32);

        self.token_hash = sha256::digest(&new_refresh_token);
        self.generation += 1;
        self.access_token_jti = access_token_jti;
        let now = chrono::Utc::now();
        self.last_used_at = now;
        self.expires_at = now.add(refresh_token_duration);

        Ok((new_refresh_token, self.expires_at))
    }
}

#[cfg(test)]
mod test {
}