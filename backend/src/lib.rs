pub mod app;

pub mod features;

pub mod domain;

pub mod util;

pub mod server;

#[cfg(test)]
mod tests {
    use reqwest::{
        Client as HttpClient,
        header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT},
    };
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::{env, error::Error, fmt, fs, path::Path};

    #[derive(Debug)]
    pub enum CopilotError {
        InvalidModel(String),
        TokenError(String),
        HttpError(String),
        Other(String),
    }

    impl fmt::Display for CopilotError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                CopilotError::InvalidModel(m) => write!(f, "Invalid model specified: {m}"),
                CopilotError::TokenError(m) => write!(f, "Token error: {m}"),
                CopilotError::HttpError(m) => write!(f, "HTTP error: {m}"),
                CopilotError::Other(m) => write!(f, "{m}"),
            }
        }
    }
    impl Error for CopilotError {}

    #[derive(Debug, Deserialize)]
    struct CopilotTokenResponse {
        token: String,
    }

    #[derive(Debug, Deserialize)]
    struct Agent {
        id: String,
        name: String,
        description: Option<String>,
    }
    #[derive(Debug, Deserialize)]
    struct AgentsResponse {
        agents: Vec<Agent>,
    }

    #[derive(Debug, Deserialize)]
    struct Model {
        id: String,
        name: String,
    }
    #[derive(Debug, Deserialize)]
    struct ModelsResponse {
        data: Vec<Model>,
    }

    #[derive(Debug, Serialize)]
    struct Message {
        role: String,
        content: String,
    }

    pub struct CopilotClient {
        http_client: HttpClient,
        github_token: String,
        editor_version: String,
    }

    impl CopilotClient {
        pub async fn from_env(editor_version: String) -> Result<Self, CopilotError> {
            let github_token =
                get_github_token().map_err(|e| CopilotError::TokenError(e.to_string()))?;
            Ok(CopilotClient {
                http_client: HttpClient::new(),
                github_token,
                editor_version,
            })
        }

        async fn get_headers(&self) -> Result<HeaderMap, CopilotError> {
            let token = self.get_copilot_token().await?;
            let mut h = HeaderMap::new();
            h.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {token}"))
                    .map_err(|e| CopilotError::Other(e.to_string()))?,
            );
            h.insert(
                "Editor-Version",
                HeaderValue::from_str(&self.editor_version)
                    .map_err(|e| CopilotError::Other(e.to_string()))?,
            );
            h.insert(
                "Editor-Plugin-Version",
                HeaderValue::from_static("CopilotChat.nvim/*"),
            );
            h.insert(
                "Copilot-Integration-Id",
                HeaderValue::from_static("vscode-chat"),
            );
            h.insert(USER_AGENT, HeaderValue::from_static("CopilotChat.nvim"));
            h.insert(ACCEPT, HeaderValue::from_static("application/json"));
            Ok(h)
        }

        async fn get_copilot_token(&self) -> Result<String, CopilotError> {
            let url = "https://api.github.com/copilot_internal/v2/token";
            let mut h = HeaderMap::new();
            h.insert(USER_AGENT, HeaderValue::from_static("CopilotChat.nvim"));
            h.insert(ACCEPT, HeaderValue::from_static("application/json"));
            h.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Token {}", self.github_token))
                    .map_err(|e| CopilotError::Other(e.to_string()))?,
            );
            let body = self
                .http_client
                .get(url)
                .headers(h)
                .send()
                .await
                .map_err(|e| CopilotError::HttpError(e.to_string()))?
                .error_for_status()
                .map_err(|e| CopilotError::HttpError(e.to_string()))?
                .text()
                .await
                .map_err(|e| CopilotError::HttpError(e.to_string()))?;
            let t: CopilotTokenResponse =
                serde_json::from_str(&body).map_err(|e| CopilotError::Other(e.to_string()))?;
            Ok(t.token)
        }

        async fn get_json(&self, url: &str) -> Result<Value, CopilotError> {
            let h = self.get_headers().await?;
            let body = self
                .http_client
                .get(url)
                .headers(h)
                .send()
                .await
                .map_err(|e| CopilotError::HttpError(e.to_string()))?
                .error_for_status()
                .map_err(|e| CopilotError::HttpError(e.to_string()))?
                .text()
                .await
                .map_err(|e| CopilotError::HttpError(e.to_string()))?;
            serde_json::from_str(&body).map_err(|e| CopilotError::Other(e.to_string()))
        }
    }

    fn get_github_token() -> Result<String, Box<dyn Error>> {
        if let Ok(token) = env::var("GITHUB_TOKEN") {
            if env::var("CODESPACES").is_ok() {
                return Ok(token);
            }
        }
        let config_dir = get_config_path()?;
        let file_paths = vec![
            format!("{config_dir}/github-copilot/hosts.json"),
            format!("{config_dir}/github-copilot/apps.json"),
        ];
        for file_path in file_paths {
            if Path::new(&file_path).exists() {
                let content = fs::read_to_string(&file_path)?;
                let json_value: Value = serde_json::from_str(&content)?;
                if let Some(obj) = json_value.as_object() {
                    for (key, value) in obj {
                        if key.contains("github.com") {
                            if let Some(oauth_token) = value.get("oauth_token") {
                                if let Some(token_str) = oauth_token.as_str() {
                                    return Ok(token_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        Err("Failed to find GitHub token".into())
    }

    /// Persist a freshly obtained Copilot OAuth token to `hosts.json` so subsequent
    /// runs of this probe (and any other CopilotChat-style tooling) can reuse it.
    fn save_github_token(token: &str) -> Result<(), Box<dyn Error>> {
        let config_dir = get_config_path()?;
        let dir = format!("{config_dir}/github-copilot");
        fs::create_dir_all(&dir)?;
        let path = format!("{dir}/hosts.json");
        let value = serde_json::json!({
            "github.com": { "oauth_token": token }
        });
        fs::write(path, serde_json::to_string_pretty(&value)?)?;
        Ok(())
    }

    /// Run GitHub's device-flow OAuth against the Copilot client_id used by
    /// the CopilotChat.nvim plugin. Returns a token usable against
    /// `/copilot_internal/v2/token`.
    async fn device_flow_login(http: &HttpClient) -> Result<String, CopilotError> {
        const CLIENT_ID: &str = "Iv1.b507a08c87ecfe98";

        let resp_body = http
            .post("https://github.com/login/device/code")
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, "CopilotChat.nvim")
            .form(&[("client_id", CLIENT_ID), ("scope", "read:user")])
            .send()
            .await
            .map_err(|e| CopilotError::HttpError(e.to_string()))?
            .error_for_status()
            .map_err(|e| CopilotError::HttpError(e.to_string()))?
            .text()
            .await
            .map_err(|e| CopilotError::HttpError(e.to_string()))?;

        let v: Value = serde_json::from_str(&resp_body)
            .map_err(|e| CopilotError::Other(format!("device/code parse: {e}: {resp_body}")))?;
        let device_code = v
            .get("device_code")
            .and_then(|x| x.as_str())
            .ok_or_else(|| CopilotError::Other("missing device_code".into()))?
            .to_string();
        let user_code = v.get("user_code").and_then(|x| x.as_str()).unwrap_or("?");
        let verification_uri = v
            .get("verification_uri")
            .and_then(|x| x.as_str())
            .unwrap_or("https://github.com/login/device");
        let interval = v.get("interval").and_then(|x| x.as_u64()).unwrap_or(5);
        let expires_in = v.get("expires_in").and_then(|x| x.as_u64()).unwrap_or(900);

        eprintln!("\n>>> GitHub Copilot device-flow auth required <<<");
        eprintln!("    Open: {verification_uri}");
        eprintln!("    Code: {user_code}");
        eprintln!(
            "    (waiting up to {}s, polling every {}s)\n",
            expires_in, interval
        );

        let deadline =
            std::time::Instant::now() + std::time::Duration::from_secs(expires_in.min(900));
        let mut delay = interval;
        loop {
            if std::time::Instant::now() >= deadline {
                return Err(CopilotError::TokenError("device flow timed out".into()));
            }
            tokio::time::sleep(std::time::Duration::from_secs(delay)).await;

            let body = http
                .post("https://github.com/login/oauth/access_token")
                .header(ACCEPT, "application/json")
                .header(USER_AGENT, "CopilotChat.nvim")
                .form(&[
                    ("client_id", CLIENT_ID),
                    ("device_code", device_code.as_str()),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()
                .await
                .map_err(|e| CopilotError::HttpError(e.to_string()))?
                .text()
                .await
                .map_err(|e| CopilotError::HttpError(e.to_string()))?;

            let v: Value = serde_json::from_str(&body)
                .map_err(|e| CopilotError::Other(format!("token parse: {e}: {body}")))?;

            if let Some(tok) = v.get("access_token").and_then(|x| x.as_str()) {
                eprintln!("    Authorized.");
                return Ok(tok.to_string());
            }
            match v.get("error").and_then(|x| x.as_str()) {
                Some("authorization_pending") => continue,
                Some("slow_down") => {
                    delay += 5;
                    continue;
                }
                Some(other) => {
                    return Err(CopilotError::TokenError(format!("device flow: {other}")));
                }
                None => {
                    return Err(CopilotError::TokenError(format!(
                        "unexpected response: {body}"
                    )));
                }
            }
        }
    }

    fn get_config_path() -> Result<String, Box<dyn Error>> {
        if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
            if !xdg.is_empty() {
                return Ok(xdg);
            }
        }
        if cfg!(target_os = "windows") {
            if let Ok(local) = env::var("LOCALAPPDATA") {
                if !local.is_empty() {
                    return Ok(local);
                }
            }
        } else if let Ok(home) = env::var("HOME") {
            return Ok(format!("{home}/.config"));
        }
        Err("Failed to find config directory".into())
    }

    #[tokio::test]
    #[ignore = "hits live Copilot API; runs device-flow if no token cached"]
    async fn probe_available_models_and_agents() {
        let _ = (
            CopilotError::InvalidModel(String::new()),
            Message {
                role: String::new(),
                content: String::new(),
            },
        );

        let token = match get_github_token() {
            Ok(t) => {
                eprintln!("Using cached Copilot OAuth token from hosts.json");
                t
            }
            Err(_) => {
                let http = HttpClient::new();
                let t = device_flow_login(&http).await.expect("device flow");
                if let Err(e) = save_github_token(&t) {
                    eprintln!("warn: failed to persist token: {e}");
                } else {
                    eprintln!("Saved token to ~/.config/github-copilot/hosts.json");
                }
                t
            }
        };

        let client = CopilotClient {
            http_client: HttpClient::new(),
            github_token: token,
            editor_version: "1.0.0".to_string(),
        };

        let raw = client
            .get_json("https://api.githubcopilot.com/models")
            .await
            .expect("fetch /models");
        let arr = raw
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();

        let mut chat: Vec<(String, String, Option<u64>, Option<u64>, Option<String>)> = Vec::new();
        let mut embed: Vec<(String, String, Option<u64>)> = Vec::new();
        let mut other: Vec<(String, String, String)> = Vec::new();

        for m in &arr {
            let id = m
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            let name = m
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            let kind = m
                .get("capabilities")
                .and_then(|c| c.get("type"))
                .and_then(|t| t.as_str())
                .unwrap_or("?")
                .to_string();
            let limits = m.get("capabilities").and_then(|c| c.get("limits"));
            let max_in = limits
                .and_then(|l| l.get("max_prompt_tokens"))
                .and_then(|v| v.as_u64());
            let max_out = limits
                .and_then(|l| l.get("max_output_tokens"))
                .and_then(|v| v.as_u64());
            let vendor = m
                .get("vendor")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            match kind.as_str() {
                "chat" => chat.push((id, name, max_in, max_out, vendor)),
                "embeddings" => embed.push((id, name, max_in)),
                k => other.push((id, name, k.to_string())),
            }
        }

        eprintln!("\n========== Chat / LLM models ({}) ==========", chat.len());
        for (id, name, mi, mo, vendor) in &chat {
            eprintln!(
                "  {:38} {:30} vendor={:?} in={:?} out={:?}",
                id, name, vendor, mi, mo
            );
        }
        eprintln!("\n========== Embedding models ({}) ==========", embed.len());
        for (id, name, mi) in &embed {
            eprintln!("  {:38} {:30} in={:?}", id, name, mi);
        }
        if !other.is_empty() {
            eprintln!("\n========== Other ({}) ==========", other.len());
            for (id, name, k) in &other {
                eprintln!("  {:38} {:30} type={}", id, name, k);
            }
        }

        match client
            .get_json("https://api.githubcopilot.com/agents")
            .await
        {
            Ok(v) => {
                let agents: AgentsResponse =
                    serde_json::from_value(v).unwrap_or(AgentsResponse { agents: Vec::new() });
                eprintln!("\n========== Agents ({}) ==========", agents.agents.len());
                for a in agents.agents {
                    eprintln!(
                        "  {:25} {} ({})",
                        a.id,
                        a.name,
                        a.description.unwrap_or_default()
                    );
                }
            }
            Err(e) => eprintln!("\nAgents endpoint error: {e}"),
        }

        // Also exercise typed deserialization just to confirm the schema lines up.
        let typed: ModelsResponse =
            serde_json::from_value(raw).expect("typed ModelsResponse parse");
        eprintln!("\n(typed parse) total models: {}", typed.data.len());
        for m in typed.data.iter().take(3) {
            eprintln!("  sample typed: id={} name={}", m.id, m.name);
        }

        assert!(!arr.is_empty(), "no models returned");
    }

    #[tokio::test]
    #[ignore = "hits live Copilot chat completions; requires cached token"]
    async fn probe_chat_three_models() {
        let token =
            get_github_token().expect("token (run probe_available_models_and_agents first)");
        let client = CopilotClient {
            http_client: HttpClient::new(),
            github_token: token,
            editor_version: "1.0.0".to_string(),
        };
        let headers = client.get_headers().await.expect("headers");

        let models = ["gpt-5.4"];
        let prompt = "Reply with exactly one short sentence saying which model you are.";

        // Look up each model's policy/state in /models so we can explain access denials.
        let raw = client
            .get_json("https://api.githubcopilot.com/models")
            .await
            .expect("/models");
        let arr = raw
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();
        for id in models {
            let entry = arr
                .iter()
                .find(|m| m.get("id").and_then(|v| v.as_str()) == Some(id));
            match entry {
                None => eprintln!("[entry] {id}: not in /models"),
                Some(m) => {
                    eprintln!(
                        "[entry] {id} =>\n{}",
                        serde_json::to_string_pretty(m).unwrap_or_default()
                    );
                }
            }
        }

        for id in models {
            eprintln!("\n--- {id} ---");
            let body = serde_json::json!({
                "model": id,
                "messages": [{"role": "user", "content": prompt}],
                "n": 1,
                "stream": false,
                "temperature": 0.0,
                "top_p": 1.0,
            });
            let started = std::time::Instant::now();
            let body_str = serde_json::to_string(&body).expect("serialize body");
            let res = client
                .http_client
                .post("https://api.githubcopilot.com/chat/completions")
                .headers(headers.clone())
                .header("Content-Type", "application/json")
                .body(body_str)
                .send()
                .await;
            match res {
                Err(e) => eprintln!("  request error: {e}"),
                Ok(r) => {
                    let status = r.status();
                    let text = r.text().await.unwrap_or_default();
                    let dt = started.elapsed();
                    if !status.is_success() {
                        eprintln!("  HTTP {status} ({:?})", dt);
                        eprintln!("  body: {}", text.chars().take(400).collect::<String>());
                        continue;
                    }
                    match serde_json::from_str::<Value>(&text) {
                        Ok(v) => {
                            let content = v
                                .get("choices")
                                .and_then(|c| c.get(0))
                                .and_then(|c| c.get("message"))
                                .and_then(|m| m.get("content"))
                                .and_then(|c| c.as_str())
                                .unwrap_or("(no content)");
                            let finish = v
                                .get("choices")
                                .and_then(|c| c.get(0))
                                .and_then(|c| c.get("finish_reason"))
                                .and_then(|f| f.as_str())
                                .unwrap_or("?");
                            let usage = v
                                .get("usage")
                                .map(|u| u.to_string())
                                .unwrap_or_else(|| "(no usage)".into());
                            let resp_model = v.get("model").and_then(|x| x.as_str()).unwrap_or("?");
                            eprintln!("  ok ({:?}) finish={finish}", dt);
                            eprintln!("  response.model: {resp_model}");
                            eprintln!("  reply: {}", content.trim());
                            eprintln!("  usage: {usage}");
                        }
                        Err(e) => {
                            eprintln!("  JSON parse error: {e}");
                            eprintln!("  body: {}", text.chars().take(400).collect::<String>());
                        }
                    }
                }
            }
        }
    }
}
