pub struct Chatbot {
    pub id: uuid::Uuid,
}

#[derive(Default)]
pub struct ChatbotOption {
    pub id: Option<uuid::Uuid>,
}

impl Chatbot {
    pub fn new(option: ChatbotOption) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
        Ok(Ok(Self {
            id: option.id.unwrap_or(uuid::Uuid::new_v4()),
        }))
    }
}
