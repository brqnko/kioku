pub struct ChatName(pub String);

impl ChatName {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 256 {
            return Err(crate::domain::DomainError::new(
                "invalid_chat_name",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatMessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: ChatMessageRole,
    pub content: String,
}

pub struct Chat {
    pub id: uuid::Uuid,
    pub name: ChatName,
    pub user_id: uuid::Uuid,
    pub messages: Vec<ChatMessage>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Default)]
pub struct ChatOption {
    pub id: Option<uuid::Uuid>,
    pub messages: Option<Vec<ChatMessage>>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_activity_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Chat {
    pub fn new(
        name: String,
        user_id: uuid::Uuid,
        option: ChatOption,
    ) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
        let name = match ChatName::new(name) {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };

        Ok(Ok(Self {
            id: option.id.unwrap_or(uuid::Uuid::new_v4()),
            name,
            user_id,
            messages: option.messages.unwrap_or_default(),
            started_at: option.started_at.unwrap_or(chrono::Utc::now()),
            last_activity_at: option.last_activity_at.unwrap_or(chrono::Utc::now()),
        }))
    }

    pub fn set_name(&mut self, name: String) -> Result<(), crate::domain::DomainError> {
        self.name = ChatName::new(name)?;
        Ok(())
    }

    pub fn add_message(&mut self, role: ChatMessageRole, content: String) {
        self.messages.push(ChatMessage { role, content });
        self.last_activity_at = chrono::Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("chat".to_string(), true)]
    #[case("a".repeat(256), true)]
    #[case("a".repeat(257), false)]
    fn chat_name_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(ChatName::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("my chat".to_string(), true, true)]
    #[case("a".repeat(257), true, false)]
    fn chat_new(#[case] name: String, #[case] outer_ok: bool, #[case] inner_ok: bool) {
        let result = Chat::new(name, uuid::Uuid::new_v4(), ChatOption::default());
        assert_eq!(result.is_ok(), outer_ok);
        if outer_ok {
            assert_eq!(result.unwrap().is_ok(), inner_ok);
        }
    }

    fn make_chat() -> Chat {
        Chat::new(
            "my chat".to_string(),
            uuid::Uuid::new_v4(),
            ChatOption::default(),
        )
        .unwrap()
        .unwrap()
    }

    #[rstest]
    #[case("new name".to_string(), true)]
    #[case("a".repeat(256), true)]
    #[case("a".repeat(257), false)]
    fn chat_set_name(#[case] input: String, #[case] should_ok: bool) {
        let mut chat = make_chat();
        assert_eq!(chat.set_name(input).is_ok(), should_ok);
    }

    #[test]
    fn chat_add_message_appends_and_updates_activity() {
        let mut chat = make_chat();
        let before = chat.last_activity_at;
        // 時刻の精度の関係で少し待機せずに検証する場合はメッセージ件数で確認
        chat.add_message(ChatMessageRole::User, "hello".to_string());
        assert_eq!(chat.messages.len(), 1);
        assert_eq!(chat.messages[0].role, ChatMessageRole::User);
        assert_eq!(chat.messages[0].content, "hello".to_string());
        assert!(chat.last_activity_at >= before);
    }

    #[test]
    fn chat_add_multiple_messages() {
        let mut chat = make_chat();
        chat.add_message(ChatMessageRole::User, "hello".to_string());
        chat.add_message(ChatMessageRole::Assistant, "hi there".to_string());
        assert_eq!(chat.messages.len(), 2);
        assert_eq!(chat.messages[1].role, ChatMessageRole::Assistant);
    }
}
