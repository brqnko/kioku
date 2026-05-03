pub struct Project {
    pub id: uuid::Uuid,
}

#[derive(Default)]
pub struct ProjectOption {
    pub id: Option<uuid::Uuid>,
}

impl Project {
    pub fn new(option: ProjectOption) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
        Ok(Ok(Self {
            id: option.id.unwrap_or(uuid::Uuid::new_v4()),
        }))
    }
}
