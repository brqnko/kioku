// create chat

pub struct CreateChatInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub name: String,
}

pub struct CreateChatOutput {
    pub chat: super::domain::Chat,
}

pub async fn create_chat(
    app: &crate::app::App,
    input: CreateChatInput,
) -> Result<Result<CreateChatOutput, crate::domain::DomainError>, anyhow::Error> {
    if !app
        .project_query_service
        .exists_owned_by_user(input.project_id, input.user_id)
        .await?
    {
        return Ok(Err(crate::domain::DomainError::new(
            "project_not_found",
            "project not found".to_string(),
            crate::domain::DomainErrorKind::NotFound,
        )));
    }

    let chat = match super::domain::Chat::new(
        input.name,
        input.user_id,
        input.project_id,
        super::domain::ChatOption::default(),
    )? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    let lock_name = format!("chat_quota:project:{}", input.project_id.as_simple());
    let mut tx = app.pool.begin().await?;
    app.locker.acquire(&mut tx, &lock_name, 10).await?;

    let work: Result<Result<(), crate::domain::DomainError>, anyhow::Error> = async {
        let existing = app
            .chat_query_service
            .count_by_project(input.project_id)
            .await?;
        if existing as usize >= super::domain::MAX_CHATS_PER_PROJECT {
            return Ok(Err(crate::domain::DomainError::new(
                "chat_per_project_quota_exceeded",
                format!(
                    "project can have at most {} chats",
                    super::domain::MAX_CHATS_PER_PROJECT
                ),
                crate::domain::DomainErrorKind::BadInput,
            )));
        }
        match app.chat_repository.save(&mut tx, &chat).await? {
            Ok(()) => Ok(Ok(())),
            Err(err) => Ok(Err(err)),
        }
    }
    .await;

    app.locker.release(&mut tx, &lock_name).await?;

    match work? {
        Ok(()) => {
            tx.commit().await?;
            Ok(Ok(CreateChatOutput { chat }))
        }
        Err(err) => Ok(Err(err)),
    }
}

// list chats

pub struct ListChatsCursor {
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub chat_id: uuid::Uuid,
}

pub struct ListChatsInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub cursor: Option<ListChatsCursor>,
    pub limit: u32,
}

pub struct ListChatsOutput {
    pub items: Vec<super::query_service::ListChatsByProjectView>,
    pub next_cursor: Option<ListChatsCursor>,
}

pub async fn list_chats(
    app: &crate::app::App,
    input: ListChatsInput,
) -> Result<Result<ListChatsOutput, crate::domain::DomainError>, anyhow::Error> {
    if !app
        .project_query_service
        .exists_owned_by_user(input.project_id, input.user_id)
        .await?
    {
        return Ok(Err(crate::domain::DomainError::new(
            "project_not_found",
            "project not found".to_string(),
            crate::domain::DomainErrorKind::NotFound,
        )));
    }

    let cursor = input
        .cursor
        .map(|c| super::query_service::ListChatsByProjectCursor {
            last_activity_at: c.last_activity_at,
            chat_id: c.chat_id,
        });

    let mut rows = app
        .chat_query_service
        .list_chats_by_project(input.project_id, cursor, input.limit + 1)
        .await?;

    let next_cursor = if rows.len() as u32 > input.limit {
        rows.pop().map(|r| ListChatsCursor {
            last_activity_at: r.last_activity_at,
            chat_id: r.id,
        })
    } else {
        None
    };

    Ok(Ok(ListChatsOutput {
        items: rows,
        next_cursor,
    }))
}

// get chat

pub struct GetChatInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub chat_id: uuid::Uuid,
}

pub struct GetChatOutput {
    pub chat: super::query_service::GetChatView,
}

pub async fn get_chat(
    app: &crate::app::App,
    input: GetChatInput,
) -> Result<Result<GetChatOutput, crate::domain::DomainError>, anyhow::Error> {
    let chat = match app.chat_query_service.get_chat(input.chat_id).await? {
        Some(ok) if ok.project_id == input.project_id => ok,
        _ => {
            return Ok(Err(crate::domain::DomainError::new(
                "chat_not_found",
                "chat not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if chat.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "chat does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    Ok(Ok(GetChatOutput { chat }))
}

// send message

pub struct SendMessageInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub chat_id: uuid::Uuid,
    pub content: String,
}

pub struct SendMessageOutput {
    pub user_message: super::domain::ChatMessage,
    pub assistant_message: super::domain::ChatMessage,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
}

pub async fn send_message(
    app: &crate::app::App,
    input: SendMessageInput,
) -> Result<Result<SendMessageOutput, crate::domain::DomainError>, anyhow::Error> {
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

    if let Err(err) = user.check_chatbot_daily_quota()? {
        return Ok(Err(err));
    }

    let mut chat = match app
        .chat_repository
        .find_for_update(&mut tx, input.chat_id)
        .await?
    {
        Some(ok) if ok.project_id == input.project_id => ok,
        _ => {
            return Ok(Err(crate::domain::DomainError::new(
                "chat_not_found",
                "chat not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if chat.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "chat does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    if chat.messages.len() + 2 > super::domain::MAX_MESSAGES_PER_CHAT {
        return Ok(Err(crate::domain::DomainError::new(
            "chat_messages_per_chat_quota_exceeded",
            format!(
                "chat can have at most {} messages",
                super::domain::MAX_MESSAGES_PER_CHAT
            ),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }

    if let Err(err) = chat.add_message(super::domain::ChatMessageRole::User, input.content) {
        return Ok(Err(err));
    }

    let completion = app
        .llm_client
        .complete(
            crate::util::llm::CopilotImpl::MODEL_GPT_5_MINI,
            chat.to_completion_input(),
        )
        .await?;

    if let Err(err) = chat.add_message(
        super::domain::ChatMessageRole::Assistant,
        completion.content,
    ) {
        return Ok(Err(err));
    }

    match app.chat_repository.save(&mut tx, &chat).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }

    user.consume_chatbot_daily_quota();
    match app.user_repository.save(&mut tx, &user).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    let len = chat.messages.len();
    let assistant_message = chat.messages[len - 1].clone();
    let user_message = chat.messages[len - 2].clone();

    Ok(Ok(SendMessageOutput {
        user_message,
        assistant_message,
        last_activity_at: chat.last_activity_at,
    }))
}

// update chat

pub struct UpdateChatInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub chat_id: uuid::Uuid,
    pub name: Option<String>,
}

pub struct UpdateChatOutput {
    pub chat: super::domain::Chat,
}

pub async fn update_chat(
    app: &crate::app::App,
    input: UpdateChatInput,
) -> Result<Result<UpdateChatOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let mut chat = match app
        .chat_repository
        .find_for_update(&mut tx, input.chat_id)
        .await?
    {
        Some(ok) if ok.project_id == input.project_id => ok,
        _ => {
            return Ok(Err(crate::domain::DomainError::new(
                "chat_not_found",
                "chat not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if chat.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "chat does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    if let Some(name) = input.name
        && let Err(err) = chat.set_name(name)
    {
        return Ok(Err(err));
    }

    match app.chat_repository.save(&mut tx, &chat).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }
    tx.commit().await?;

    Ok(Ok(UpdateChatOutput { chat }))
}

// remove chat

pub struct RemoveChatInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub chat_id: uuid::Uuid,
}

pub struct RemoveChatOutput {}

pub async fn remove_chat(
    app: &crate::app::App,
    input: RemoveChatInput,
) -> Result<Result<RemoveChatOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let chat = match app
        .chat_repository
        .find_for_update(&mut tx, input.chat_id)
        .await?
    {
        Some(ok) if ok.project_id == input.project_id => ok,
        _ => {
            return Ok(Err(crate::domain::DomainError::new(
                "chat_not_found",
                "chat not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if chat.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "chat does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    match app.chat_repository.remove(&mut tx, chat.id).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }
    tx.commit().await?;

    Ok(Ok(RemoveChatOutput {}))
}
