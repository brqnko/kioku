pub struct Podcast {
    pub id: uuid::Uuid,
}

#[derive(Default)]
pub struct PodcastOption {
    pub id: Option<uuid::Uuid>,
}

impl Podcast {
    pub fn new(option: PodcastOption) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
        Ok(Ok(Self {
            id: option.id.unwrap_or(uuid::Uuid::new_v4()),
        }))
    }
}
