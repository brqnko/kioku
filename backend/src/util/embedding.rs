#[async_trait::async_trait]
pub trait EmbeddingClient: Send + Sync {
    async fn embed(
        &self,
        inputs: std::collections::HashMap<uuid::Uuid, String>,
    ) -> Result<std::collections::HashMap<uuid::Uuid, Vec<f32>>, anyhow::Error>;
}

#[derive(Default)]
pub struct EmbeddingClientImpl;

impl EmbeddingClientImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl EmbeddingClient for EmbeddingClientImpl {
    async fn embed(
        &self,
        _inputs: std::collections::HashMap<uuid::Uuid, String>,
    ) -> Result<std::collections::HashMap<uuid::Uuid, Vec<f32>>, anyhow::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[derive(serde::Deserialize)]
    struct TokenResponse {
        token: String,
    }

    #[derive(serde::Serialize)]
    struct EmbeddingRequest<'a> {
        model: &'a str,
        input: Vec<&'a str>,
    }

    #[derive(serde::Deserialize)]
    struct EmbeddingResponse {
        data: Vec<EmbeddingItem>,
    }

    #[derive(serde::Deserialize)]
    struct EmbeddingItem {
        embedding: Vec<f32>,
    }

    async fn copilot_token(http: &reqwest::Client, github_token: &str) -> String {
        http.get("https://api.github.com/copilot_internal/v2/token")
            .header(reqwest::header::USER_AGENT, "kioku")
            .header(reqwest::header::ACCEPT, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Token {github_token}"),
            )
            .send()
            .await
            .expect("request copilot token")
            .error_for_status()
            .expect("copilot token status")
            .json::<TokenResponse>()
            .await
            .expect("decode copilot token")
            .token
    }

    async fn embed_batch(
        http: &reqwest::Client,
        copilot_token: &str,
        inputs: Vec<&str>,
    ) -> Vec<Vec<f32>> {
        let req = EmbeddingRequest {
            model: "text-embedding-3-small",
            input: inputs,
        };

        let resp = http
            .post("https://api.githubcopilot.com/embeddings")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {copilot_token}"),
            )
            .header("Editor-Version", "1.0.0")
            .header("Editor-Plugin-Version", "kioku/*")
            .header("Copilot-Integration-Id", "vscode-chat")
            .header(reqwest::header::USER_AGENT, "kioku")
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&req)
            .send()
            .await
            .expect("post embeddings");

        let status = resp.status();
        let body = resp.text().await.expect("read body");
        assert!(status.is_success(), "embeddings failed: {status} {body}");

        let parsed: EmbeddingResponse =
            serde_json::from_str(&body).expect("decode embeddings response");
        parsed.data.into_iter().map(|d| d.embedding).collect()
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len(), "vector length mismatch");
        let mut dot = 0.0_f32;
        let mut na = 0.0_f32;
        let mut nb = 0.0_f32;
        for i in 0..a.len() {
            dot += a[i] * b[i];
            na += a[i] * a[i];
            nb += b[i] * b[i];
        }
        dot / (na.sqrt() * nb.sqrt())
    }

    #[tokio::test]
    async fn embed_5g_via_copilot_text_embedding_3_small() {
        let github_token =
            std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set for this test");

        let http = reqwest::Client::new();
        let token = copilot_token(&http, &github_token).await;

        let input_text = "5Gは体にとって有害だ";
        let embeddings = embed_batch(&http, &token, vec![input_text]).await;
        let embedding = &embeddings[0];

        println!("input    : {input_text}");
        println!("model    : text-embedding-3-small");
        println!("dim      : {}", embedding.len());
        println!("first 8  : {:?}", &embedding[..8.min(embedding.len())]);
        println!("last 8   : {:?}", &embedding[embedding.len().saturating_sub(8)..]);

        assert!(!embedding.is_empty(), "embedding vector is empty");
    }

    #[tokio::test]
    async fn cosine_similarity_role_and_polarity() {
        let github_token =
            std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set for this test");

        let http = reqwest::Client::new();
        let token = copilot_token(&http, &github_token).await;

        let labels = ["A", "B", "C", "D", "E"];
        let texts = [
            "BobはAliceを好きだ",         // A: Bob -> Alice
            "AliceはBobを好きだ",         // B: Alice -> Bob (逆方向)
            "BobはAliceに好かれている",   // C: Alice -> Bob (B の受動態 = 同義)
            "BobはCarolを好きだ",         // D: 対象を入れ替え
            "BobはAliceを嫌いだ",         // E: 極性反転
        ];

        let embeddings = embed_batch(&http, &token, texts.to_vec()).await;

        println!("model    : text-embedding-3-small");
        println!("dim      : {}", embeddings[0].len());
        for (l, t) in labels.iter().zip(texts.iter()) {
            println!("{l}        : {t}");
        }
        println!();

        println!("         {}", labels.iter().map(|l| format!("    {l}   ")).collect::<String>());
        for i in 0..texts.len() {
            let mut row = format!("   {}    ", labels[i]);
            for j in 0..texts.len() {
                let s = cosine_similarity(&embeddings[i], &embeddings[j]);
                row.push_str(&format!("{s:>8.4}"));
            }
            println!("{row}");
        }
        println!();

        let key_pairs = [
            ("A-B (Bob→Alice vs Alice→Bob, 役割反転)", 0, 1),
            ("A-C (Bob→Alice vs Alice→Bob受動, 役割反転)", 0, 2),
            ("B-C (Alice→Bob vs その受動態, 言い換え)", 1, 2),
            ("A-D (相手をAlice→Carolに置換)", 0, 3),
            ("A-E (好き→嫌い, 極性反転)", 0, 4),
        ];
        for (label, i, j) in key_pairs {
            let s = cosine_similarity(&embeddings[i], &embeddings[j]);
            println!("cos({label:<48}) = {s:.4}");
        }
    }

    #[tokio::test]
    async fn cosine_similarity_5g_vs_weather() {
        let github_token =
            std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set for this test");

        let http = reqwest::Client::new();
        let token = copilot_token(&http, &github_token).await;

        let a = "5Gは人体にとって有害だ";
        let b = "5Gは高速で画期的な技術だ";
        let c = "今日は天気がいいですね。";

        let embeddings = embed_batch(&http, &token, vec![a, b, c]).await;
        let sim_ab = cosine_similarity(&embeddings[0], &embeddings[1]);
        let sim_ac = cosine_similarity(&embeddings[0], &embeddings[2]);
        let sim_bc = cosine_similarity(&embeddings[1], &embeddings[2]);

        println!("model    : text-embedding-3-small");
        println!("dim      : {}", embeddings[0].len());
        println!("A        : {a}");
        println!("B        : {b}");
        println!("C        : {c}");
        println!("cos(A,B) : {sim_ab:.6}");
        println!("cos(A,C) : {sim_ac:.6}");
        println!("cos(B,C) : {sim_bc:.6}");

        for s in [sim_ab, sim_ac, sim_bc] {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[tokio::test]
    async fn cosine_similarity_5g_harmful_vs_innovative() {
        let github_token =
            std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set for this test");

        let http = reqwest::Client::new();
        let token = copilot_token(&http, &github_token).await;

        let a = "5Gは人体にとって有害だ";
        let b = "5Gは高速で画期的な技術だ";

        let embeddings = embed_batch(&http, &token, vec![a, b]).await;
        let sim = cosine_similarity(&embeddings[0], &embeddings[1]);

        println!("model    : text-embedding-3-small");
        println!("dim      : {}", embeddings[0].len());
        println!("text A   : {a}");
        println!("text B   : {b}");
        println!("cos sim  : {sim:.6}");

        assert!((-1.0..=1.0).contains(&sim));
    }
}
