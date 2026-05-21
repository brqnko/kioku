#[derive(Clone, Copy)]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Role {
    fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

pub struct Message {
    pub role: Role,
    pub content: String,
}

pub struct CompletionInput {
    pub messages: Vec<Message>,
}

pub struct CompletionOutput {
    pub content: String,
}

#[async_trait::async_trait]
pub trait LLMClient: Send + Sync {
    async fn complete(
        &self,
        model: &'static str,
        input: CompletionInput,
    ) -> Result<CompletionOutput, anyhow::Error>;
}

// Copilot implementation

#[derive(Clone)]
struct CachedCopilotToken {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct CopilotImpl {
    http_client: reqwest::Client,
    github_token: String,
    editor_version: String,
    cached_token: std::sync::Mutex<Option<CachedCopilotToken>>,
}

impl CopilotImpl {
    pub const MODEL_GEMINI_3_1_PRO: &'static str = "gemini-3.1-pro-preview";
    pub const MODEL_GPT_5: &'static str = "gpt-5.2";
    pub const MODEL_GPT_5_MINI: &'static str = "gpt-5-mini";
    pub const MODEL_GPT_5_4_MINI: &'static str = "gpt-5.4-mini";

    pub fn new(github_token: String) -> Result<Self, anyhow::Error> {
        Ok(Self {
            http_client: reqwest::Client::builder().build()?,
            github_token,
            editor_version: "1.0.0".to_string(),
            cached_token: std::sync::Mutex::new(None),
        })
    }

    async fn copilot_token(&self) -> Result<String, anyhow::Error> {
        #[derive(serde::Deserialize)]
        struct TokenResponse {
            token: String,
            expires_at: i64,
        }

        {
            let cache = self.cached_token.lock().unwrap();
            if let Some(c) = cache.as_ref()
                && c.expires_at > chrono::Utc::now() + chrono::Duration::seconds(60)
            {
                return Ok(c.token.clone());
            }
        }

        let mut h = reqwest::header::HeaderMap::new();
        h.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("kioku"));
        h.insert(reqwest::header::ACCEPT, reqwest::header::HeaderValue::from_static("application/json"));
        h.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Token {}", self.github_token))?,
        );

        let resp = self
            .http_client
            .get("https://api.github.com/copilot_internal/v2/token")
            .headers(h)
            .send()
            .await?
            .error_for_status()?
            .json::<TokenResponse>()
            .await?;

        let expires_at = chrono::DateTime::<chrono::Utc>::from_timestamp(resp.expires_at, 0)
            .ok_or_else(|| anyhow::anyhow!("invalid copilot expires_at: {}", resp.expires_at))?;

        {
            let mut cache = self.cached_token.lock().unwrap();
            *cache = Some(CachedCopilotToken { token: resp.token.clone(), expires_at });
        }

        Ok(resp.token)
    }

    async fn build_headers(&self) -> Result<reqwest::header::HeaderMap, anyhow::Error> {
        let token = self.copilot_token().await?;
        let mut h = reqwest::header::HeaderMap::new();
        h.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))?);
        h.insert("Editor-Version", reqwest::header::HeaderValue::from_str(&self.editor_version)?);
        h.insert("Editor-Plugin-Version", reqwest::header::HeaderValue::from_static("kioku/*"));
        h.insert("Copilot-Integration-Id", reqwest::header::HeaderValue::from_static("vscode-chat"));
        h.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("kioku"));
        h.insert(reqwest::header::ACCEPT, reqwest::header::HeaderValue::from_static("application/json"));
        Ok(h)
    }
}

#[async_trait::async_trait]
impl LLMClient for CopilotImpl {
    async fn complete(
        &self,
        model: &'static str,
        input: CompletionInput,
    ) -> Result<CompletionOutput, anyhow::Error> {
        #[derive(serde::Serialize)]
        struct MessageBody<'a> { role: &'a str, content: &'a str }
        #[derive(serde::Serialize)]
        struct ChatRequest<'a> { model: &'a str, messages: Vec<MessageBody<'a>>, n: u32, stream: bool }
        #[derive(serde::Deserialize)]
        struct ChatResponse { choices: Vec<ChatChoice> }
        #[derive(serde::Deserialize)]
        struct ChatChoice { message: ChatMessage }
        #[derive(serde::Deserialize)]
        struct ChatMessage { content: String }

        let messages = input.messages.iter().map(|m| MessageBody { role: m.role.as_str(), content: &m.content }).collect::<Vec<_>>();
        let req = ChatRequest { model, messages, n: 1, stream: false };

        let headers = self.build_headers().await?;
        let raw = self
            .http_client
            .post("https://api.githubcopilot.com/chat/completions")
            .headers(headers)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&req)
            .send()
            .await?;

        if let Err(e) = raw.error_for_status_ref() {
            let retry_after = raw.headers().get("retry-after")
                .or_else(|| raw.headers().get("x-ratelimit-reset-requests"))
                .and_then(|v| v.to_str().ok())
                .map(|s| format!(", retry after: {s}"))
                .unwrap_or_default();
            return Err(anyhow::anyhow!("{e}{retry_after}"));
        }

        let content = raw.json::<ChatResponse>().await?
            .choices.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("no choices in copilot response"))?
            .message.content;

        Ok(CompletionOutput { content })
    }
}

