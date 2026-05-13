pub const MAX_CODE_LEN: usize = 64 * 1024;
pub const MAX_STDIN_LEN: usize = 32 * 1024;
pub const MAX_OPTION_LEN: usize = 4 * 1024;
pub const MAX_COMPILER_LEN: usize = 64;

pub struct RunRequest {
    pub code: String,
    pub compiler: String,
    pub stdin: Option<String>,
    pub compiler_options: Option<String>,
    pub compiler_option_raw: Option<String>,
    pub runtime_option_raw: Option<String>,
}

pub struct RunResponse {
    pub status: Option<String>,
    pub signal: Option<String>,
    pub compiler_output: Option<String>,
    pub compiler_error: Option<String>,
    pub compiler_message: Option<String>,
    pub program_output: Option<String>,
    pub program_error: Option<String>,
    pub program_message: Option<String>,
}

#[derive(Debug)]
pub enum CodeRunnerError {
    Upstream(anyhow::Error),
    Rejected(String),
}

impl std::fmt::Display for CodeRunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Upstream(e) => write!(f, "upstream error: {e}"),
            Self::Rejected(msg) => write!(f, "rejected by upstream: {msg}"),
        }
    }
}

impl std::error::Error for CodeRunnerError {}

#[derive(Clone)]
pub struct CompilerSummary {
    pub name: String,
    pub language: String,
    pub display_name: String,
    pub version: String,
}

#[async_trait::async_trait]
pub trait CodeRunnerClient: Send + Sync {
    async fn run(&self, request: RunRequest) -> Result<RunResponse, CodeRunnerError>;
    async fn list_compilers(&self) -> Result<Vec<CompilerSummary>, CodeRunnerError>;
}

const WANDBOX_URL: &str = "https://wandbox.org/api/compile.json";
const WANDBOX_LIST_URL: &str = "https://wandbox.org/api/list.json";
const REQUEST_TIMEOUT_SECS: u64 = 600;
const COMPILER_LIST_TTL_SECS: u64 = 24 * 60 * 60;

pub struct WandboxClient {
    http_client: reqwest::Client,
    compiler_cache: tokio::sync::RwLock<Option<(std::time::Instant, Vec<CompilerSummary>)>>,
}

impl WandboxClient {
    pub fn new() -> Result<Self, anyhow::Error> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()?;
        Ok(Self {
            http_client,
            compiler_cache: tokio::sync::RwLock::new(None),
        })
    }

    async fn fetch_compilers(&self) -> Result<Vec<CompilerSummary>, anyhow::Error> {
        let resp = self
            .http_client
            .get(WANDBOX_LIST_URL)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow::anyhow!("wandbox list responded with {status}"));
        }

        let raw: Vec<WandboxCompiler> = resp.json().await?;
        Ok(raw
            .into_iter()
            .map(|c| CompilerSummary {
                name: c.name,
                language: c.language,
                display_name: c.display_name,
                version: c.version,
            })
            .collect())
    }
}

#[derive(serde::Deserialize)]
struct WandboxCompiler {
    name: String,
    language: String,
    #[serde(rename = "display-name", default)]
    display_name: String,
    #[serde(default)]
    version: String,
}

#[derive(serde::Serialize)]
struct WandboxRequest<'a> {
    code: &'a str,
    compiler: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    stdin: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<&'a str>,
    #[serde(rename = "compiler-option-raw", skip_serializing_if = "Option::is_none")]
    compiler_option_raw: Option<&'a str>,
    #[serde(rename = "runtime-option-raw", skip_serializing_if = "Option::is_none")]
    runtime_option_raw: Option<&'a str>,
    save: bool,
}

#[derive(serde::Deserialize)]
struct WandboxResponse {
    status: Option<String>,
    signal: Option<String>,
    compiler_output: Option<String>,
    compiler_error: Option<String>,
    compiler_message: Option<String>,
    program_output: Option<String>,
    program_error: Option<String>,
    program_message: Option<String>,
}

#[async_trait::async_trait]
impl CodeRunnerClient for WandboxClient {
    async fn run(&self, request: RunRequest) -> Result<RunResponse, CodeRunnerError> {
        let RunRequest {
            code,
            compiler,
            stdin,
            compiler_options,
            compiler_option_raw,
            runtime_option_raw,
        } = request;

        let body = WandboxRequest {
            code: &code,
            compiler: &compiler,
            stdin: stdin.as_deref(),
            options: compiler_options.as_deref(),
            compiler_option_raw: compiler_option_raw.as_deref(),
            runtime_option_raw: runtime_option_raw.as_deref(),
            save: false,
        };

        let code_len = code.len();
        let stdin_len = stdin.as_deref().map(str::len).unwrap_or(0);

        tracing::info!(
            target: "code_runner",
            compiler = %compiler,
            code_len,
            stdin_len,
            "wandbox compile request"
        );

        let resp = self
            .http_client
            .post(WANDBOX_URL)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::ACCEPT, "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| CodeRunnerError::Upstream(e.into()))?;

        let status = resp.status();
        if status.is_client_error() {
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!(
                target: "code_runner",
                compiler = %compiler,
                http_status = %status,
                "wandbox rejected request"
            );
            let trimmed: String = text.chars().take(512).collect();
            return Err(CodeRunnerError::Rejected(trimmed));
        }
        if !status.is_success() {
            tracing::warn!(
                target: "code_runner",
                compiler = %compiler,
                http_status = %status,
                "wandbox upstream failure"
            );
            return Err(CodeRunnerError::Upstream(anyhow::anyhow!(
                "wandbox responded with {status}"
            )));
        }

        let parsed: WandboxResponse = resp
            .json()
            .await
            .map_err(|e| CodeRunnerError::Upstream(e.into()))?;

        tracing::info!(
            target: "code_runner",
            compiler = %compiler,
            status = parsed.status.as_deref().unwrap_or(""),
            "wandbox compile success"
        );

        Ok(RunResponse {
            status: parsed.status,
            signal: parsed.signal,
            compiler_output: parsed.compiler_output,
            compiler_error: parsed.compiler_error,
            compiler_message: parsed.compiler_message,
            program_output: parsed.program_output,
            program_error: parsed.program_error,
            program_message: parsed.program_message,
        })
    }

    async fn list_compilers(&self) -> Result<Vec<CompilerSummary>, CodeRunnerError> {
        {
            let cache = self.compiler_cache.read().await;
            if let Some((fetched_at, list)) = cache.as_ref()
                && fetched_at.elapsed().as_secs() < COMPILER_LIST_TTL_SECS
            {
                return Ok(list.clone());
            }
        }

        let mut cache = self.compiler_cache.write().await;
        if let Some((fetched_at, list)) = cache.as_ref()
            && fetched_at.elapsed().as_secs() < COMPILER_LIST_TTL_SECS
        {
            return Ok(list.clone());
        }

        match self.fetch_compilers().await {
            Ok(list) => {
                *cache = Some((std::time::Instant::now(), list.clone()));
                Ok(list)
            }
            Err(err) => {
                tracing::warn!(
                    target: "code_runner",
                    error = %err,
                    "wandbox list refresh failed"
                );
                if let Some((_, stale)) = cache.as_ref() {
                    return Ok(stale.clone());
                }
                Err(CodeRunnerError::Upstream(err))
            }
        }
    }
}
