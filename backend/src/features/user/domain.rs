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
            ));
        }

        Ok(Self(string))
    }
}

pub struct LanguageCode(pub String);

impl LanguageCode {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 7 {
            return Err(crate::domain::DomainError::new(
                "invalid_language_code",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

pub struct Iss(pub String);

impl Iss {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if string.is_empty() {
            return Err(crate::domain::DomainError::new(
                "invalid_iss",
                "iss must not be empty".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }
        if len > 255 {
            return Err(crate::domain::DomainError::new(
                "invalid_iss",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

pub struct Sub(pub String);

impl Sub {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if string.is_empty() {
            return Err(crate::domain::DomainError::new(
                "invalid_sub",
                "sub must not be empty".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }
        if len > 255 {
            return Err(crate::domain::DomainError::new(
                "invalid_sub",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

pub struct User {
    pub id: uuid::Uuid,
    pub display_name: DisplayName,
    pub language_code: LanguageCode,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub iss: Iss,
    pub sub: Sub,
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
    pub const MAX_RECENT_SEEN_FILE_IDS: usize = 4;

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
        let iss = match Iss::new(iss) {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };
        let sub = match Sub::new(sub) {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };

        if let Some(recent_file_ids) = &option.recent_seen_file_ids
            && recent_file_ids.len() > Self::MAX_RECENT_SEEN_FILE_IDS
        {
            return Err(anyhow::anyhow!(
                "recent_file_ids len is too long: {}",
                recent_file_ids.len()
            ));
        }

        if let Some(ai_learning_summary) = &option.ai_learning_summary
            && ai_learning_summary.len() > 512
        {
            return Err(anyhow::anyhow!(
                "ai learning summary len is too long: {}",
                ai_learning_summary.len()
            ));
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
            ai_learning_summary_updated_at: option
                .ai_learning_summary_updated_at
                .unwrap_or(chrono::Utc::now()),
        }))
    }

    pub fn set_display_name(
        &mut self,
        display_name: String,
    ) -> Result<(), crate::domain::DomainError> {
        self.display_name = DisplayName::new(display_name)?;
        Ok(())
    }

    pub fn set_language_code(
        &mut self,
        language_code: String,
    ) -> Result<(), crate::domain::DomainError> {
        self.language_code = LanguageCode::new(language_code)?;
        Ok(())
    }

    pub fn push_recent_seen_file_id(
        &mut self,
        file_id: uuid::Uuid,
    ) -> Result<(), crate::domain::DomainError> {
        self.recent_seen_file_ids.retain(|id| *id != file_id);
        self.recent_seen_file_ids.insert(0, file_id);
        self.recent_seen_file_ids
            .truncate(Self::MAX_RECENT_SEEN_FILE_IDS);
        Ok(())
    }

    pub fn set_ai_learning_summary(&mut self, summary: String) -> Result<(), anyhow::Error> {
        let len = summary.len();
        if len > 512 {
            return Err(anyhow::anyhow!(
                "ai learning summary len is too long: {len}"
            ));
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
    pub fn new(
        user_id: uuid::Uuid,
        ip_address: String,
        user_agent: String,
        option: RefreshTokenOption,
    ) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
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

    pub fn rotate(
        &mut self,
        refresh_token_duration: chrono::Duration,
        access_token_jti: uuid::Uuid,
    ) -> Result<(String, chrono::DateTime<chrono::Utc>), anyhow::Error> {
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
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("hello".to_string(), true)]
    #[case("a".repeat(32), true)]
    #[case("a".repeat(33), false)]
    fn display_name_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(DisplayName::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("en".to_string(), true)]
    #[case("a".repeat(7), true)]
    #[case("a".repeat(8), false)]
    fn language_code_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(LanguageCode::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("https://accounts.google.com".to_string(), true)]
    #[case("a".repeat(255), true)]
    #[case("a".repeat(256), false)]
    #[case("".to_string(), false)]
    fn iss_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(Iss::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("1234567890".to_string(), true)]
    #[case("a".repeat(255), true)]
    #[case("a".repeat(256), false)]
    #[case("".to_string(), false)]
    fn sub_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(Sub::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("alice".to_string(), "en".to_string(), "iss".to_string(), "sub".to_string(), UserOption::default(), true, true)]
    #[case("a".repeat(33), "en".to_string(), "iss".to_string(), "sub".to_string(), UserOption::default(), true, false)]
    #[case("alice".to_string(), "a".repeat(8), "iss".to_string(), "sub".to_string(), UserOption::default(), true, false)]
    #[case("alice".to_string(), "en".to_string(), "".to_string(), "sub".to_string(), UserOption::default(), true, false)]
    #[case("alice".to_string(), "en".to_string(), "a".repeat(256), "sub".to_string(), UserOption::default(), true, false)]
    #[case("alice".to_string(), "en".to_string(), "iss".to_string(), "".to_string(), UserOption::default(), true, false)]
    #[case("alice".to_string(), "en".to_string(), "iss".to_string(), "a".repeat(256), UserOption::default(), true, false)]
    #[case(
        "alice".to_string(),
        "en".to_string(),
        "iss".to_string(),
        "sub".to_string(),
        UserOption { recent_seen_file_ids: Some(vec![uuid::Uuid::new_v4(); 5]), ..Default::default() },
        false,
        false
    )]
    #[case(
        "alice".to_string(),
        "en".to_string(),
        "iss".to_string(),
        "sub".to_string(),
        UserOption { ai_learning_summary: Some("a".repeat(513)), ..Default::default() },
        false,
        false
    )]
    fn user_new(
        #[case] display_name: String,
        #[case] language_code: String,
        #[case] iss: String,
        #[case] sub: String,
        #[case] option: UserOption,
        #[case] outer_ok: bool,
        #[case] inner_ok: bool,
    ) {
        let result = User::new(display_name, language_code, iss, sub, option);
        assert_eq!(result.is_ok(), outer_ok);
        if outer_ok {
            assert_eq!(result.unwrap().is_ok(), inner_ok);
        }
    }

    fn make_user() -> User {
        User::new(
            "testuser".to_string(),
            "en".to_string(),
            "iss".to_string(),
            "sub".to_string(),
            UserOption::default(),
        )
        .unwrap()
        .unwrap()
    }

    #[rstest]
    #[case("bob".to_string(), true)]
    #[case("a".repeat(32), true)]
    #[case("a".repeat(33), false)]
    fn user_set_display_name(#[case] input: String, #[case] should_ok: bool) {
        let mut user = make_user();
        assert_eq!(user.set_display_name(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("ja".to_string(), true)]
    #[case("a".repeat(7), true)]
    #[case("a".repeat(8), false)]
    fn user_set_language_code(#[case] input: String, #[case] should_ok: bool) {
        let mut user = make_user();
        assert_eq!(user.set_language_code(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("summary".to_string(), true)]
    #[case("a".repeat(512), true)]
    #[case("a".repeat(513), false)]
    fn user_set_ai_learning_summary(#[case] input: String, #[case] should_ok: bool) {
        let mut user = make_user();
        assert_eq!(user.set_ai_learning_summary(input).is_ok(), should_ok);
    }

    #[test]
    fn push_recent_seen_file_id_prepends_to_front() {
        let mut user = make_user();
        let a = uuid::Uuid::new_v4();
        let b = uuid::Uuid::new_v4();
        user.push_recent_seen_file_id(a).unwrap();
        user.push_recent_seen_file_id(b).unwrap();
        assert_eq!(user.recent_seen_file_ids, vec![b, a]);
    }

    #[test]
    fn push_recent_seen_file_id_dedupes_existing() {
        let mut user = make_user();
        let a = uuid::Uuid::new_v4();
        let b = uuid::Uuid::new_v4();
        user.push_recent_seen_file_id(a).unwrap();
        user.push_recent_seen_file_id(b).unwrap();
        user.push_recent_seen_file_id(a).unwrap();
        assert_eq!(user.recent_seen_file_ids, vec![a, b]);
    }

    #[test]
    fn push_recent_seen_file_id_truncates_to_max() {
        let mut user = make_user();
        let ids: Vec<_> = (0..User::MAX_RECENT_SEEN_FILE_IDS + 2)
            .map(|_| uuid::Uuid::new_v4())
            .collect();
        for id in &ids {
            user.push_recent_seen_file_id(*id).unwrap();
        }
        assert_eq!(
            user.recent_seen_file_ids.len(),
            User::MAX_RECENT_SEEN_FILE_IDS
        );
        let expected: Vec<_> = ids
            .iter()
            .rev()
            .take(User::MAX_RECENT_SEEN_FILE_IDS)
            .copied()
            .collect();
        assert_eq!(user.recent_seen_file_ids, expected);
    }

    fn make_refresh_token() -> RefreshToken {
        RefreshToken::new(
            uuid::Uuid::new_v4(),
            "127.0.0.1".to_string(),
            "Mozilla/5.0".to_string(),
            RefreshTokenOption::default(),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn refresh_token_rotate_increments_generation() {
        let mut token = make_refresh_token();
        let initial_gen = token.generation;
        token
            .rotate(chrono::Duration::days(7), uuid::Uuid::new_v4())
            .unwrap();
        assert_eq!(token.generation, initial_gen + 1);
    }

    #[test]
    fn refresh_token_rotate_updates_token_hash() {
        let mut token = make_refresh_token();
        let (new_raw_token, _) = token
            .rotate(chrono::Duration::days(7), uuid::Uuid::new_v4())
            .unwrap();
        assert_eq!(token.token_hash, sha256::digest(&new_raw_token));
    }

    #[test]
    fn refresh_token_rotate_updates_jti() {
        let mut token = make_refresh_token();
        let new_jti = uuid::Uuid::new_v4();
        token.rotate(chrono::Duration::days(7), new_jti).unwrap();
        assert_eq!(token.access_token_jti, new_jti);
    }

    #[test]
    fn refresh_token_rotate_returns_correct_expires_at() {
        let mut token = make_refresh_token();
        let (_, returned_expires_at) = token
            .rotate(chrono::Duration::days(7), uuid::Uuid::new_v4())
            .unwrap();
        assert_eq!(token.expires_at, returned_expires_at);
    }
}
