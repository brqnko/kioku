pub const MAX_PODCASTS_PER_PROJECT: usize = 50;

pub const VOICE_STYLES: &[&str] = &["female", "male"];

pub struct VoiceStyle(#[allow(dead_code)] pub String);

impl VoiceStyle {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        if !VOICE_STYLES.contains(&string.as_str()) {
            return Err(crate::domain::DomainError::new(
                "invalid_voice_style",
                format!("unknown voice style: {string}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }
        Ok(Self(string))
    }
}

pub struct PodcastName(pub String);

impl PodcastName {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 256 {
            return Err(crate::domain::DomainError::new(
                "invalid_podcast_name",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PodcastLength {
    Short,
    Normal,
    Long,
}

impl PodcastLength {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::Normal => "normal",
            Self::Long => "long",
        }
    }

    pub fn new(string: &str) -> Result<Self, crate::domain::DomainError> {
        match string {
            "short" => Ok(Self::Short),
            "normal" => Ok(Self::Normal),
            "long" => Ok(Self::Long),
            _ => Err(crate::domain::DomainError::new(
                "invalid_podcast_length",
                format!("unknown podcast length: {string}"),
                crate::domain::DomainErrorKind::BadInput,
            )),
        }
    }
}

pub struct PodcastDescription(pub String);

impl PodcastDescription {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 1024 {
            return Err(crate::domain::DomainError::new(
                "invalid_podcast_description",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PodcastScriptEntry {
    pub speaker: String,
    pub text: String,
}

pub struct Podcast {
    pub id: uuid::Uuid,
    pub name: PodcastName,
    pub description: PodcastDescription,
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub used_file_ids: Vec<uuid::Uuid>,
    pub podcast_script: Vec<PodcastScriptEntry>,
    pub audio_storage_id: uuid::Uuid,
    pub podcast_created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Default)]
pub struct PodcastOption {
    pub id: Option<uuid::Uuid>,
    pub podcast_created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Podcast {
    pub fn set_name(&mut self, name: String) -> Result<(), crate::domain::DomainError> {
        self.name = PodcastName::new(name)?;
        Ok(())
    }

    pub fn set_description(
        &mut self,
        description: String,
    ) -> Result<(), crate::domain::DomainError> {
        self.description = PodcastDescription::new(description)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        description: String,
        user_id: uuid::Uuid,
        project_id: uuid::Uuid,
        used_file_ids: Vec<uuid::Uuid>,
        podcast_script: Vec<PodcastScriptEntry>,
        audio_storage_id: uuid::Uuid,
        option: PodcastOption,
    ) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
        let name = match PodcastName::new(name) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        };
        let description = match PodcastDescription::new(description) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        };

        Ok(Ok(Self {
            id: option.id.unwrap_or(uuid::Uuid::new_v4()),
            name,
            description,
            user_id,
            project_id,
            used_file_ids,
            podcast_script,
            audio_storage_id,
            podcast_created_at: option.podcast_created_at.unwrap_or(chrono::Utc::now()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("podcast".to_string(), true)]
    #[case("a".repeat(256), true)]
    #[case("a".repeat(257), false)]
    fn podcast_name_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(PodcastName::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("desc".to_string(), true)]
    #[case("a".repeat(1024), true)]
    #[case("a".repeat(1025), false)]
    fn podcast_description_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(PodcastDescription::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("my podcast".to_string(), "desc".to_string(), true, true)]
    #[case("a".repeat(257), "desc".to_string(), true, false)]
    #[case("my podcast".to_string(), "a".repeat(1025), true, false)]
    fn podcast_new(
        #[case] name: String,
        #[case] description: String,
        #[case] outer_ok: bool,
        #[case] inner_ok: bool,
    ) {
        let result = Podcast::new(
            name,
            description,
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            vec![],
            vec![],
            uuid::Uuid::new_v4(),
            PodcastOption::default(),
        );
        assert_eq!(result.is_ok(), outer_ok);
        if outer_ok {
            assert_eq!(result.unwrap().is_ok(), inner_ok);
        }
    }

    #[test]
    fn podcast_new_stores_used_file_ids() {
        let file_ids = vec![uuid::Uuid::new_v4(), uuid::Uuid::new_v4()];
        let podcast = Podcast::new(
            "my podcast".to_string(),
            "desc".to_string(),
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            file_ids.clone(),
            vec![],
            uuid::Uuid::new_v4(),
            PodcastOption::default(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(podcast.used_file_ids, file_ids);
    }

    #[test]
    fn podcast_new_stores_script_entries() {
        let script = vec![
            PodcastScriptEntry {
                speaker: "Alice".to_string(),
                text: "Hello".to_string(),
            },
            PodcastScriptEntry {
                speaker: "Bob".to_string(),
                text: "Hi there".to_string(),
            },
        ];
        let podcast = Podcast::new(
            "my podcast".to_string(),
            "desc".to_string(),
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            vec![],
            script,
            uuid::Uuid::new_v4(),
            PodcastOption::default(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(podcast.podcast_script.len(), 2);
        assert_eq!(podcast.podcast_script[0].speaker, "Alice");
        assert_eq!(podcast.podcast_script[1].text, "Hi there");
    }
}
