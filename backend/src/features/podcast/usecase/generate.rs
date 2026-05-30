pub struct GeneratePodcastInput {
    pub podcast_id: uuid::Uuid,
}

pub struct GeneratePodcastOutput {}

pub async fn generate_podcast(
    app: &crate::app::App,
    input: GeneratePodcastInput,
) -> Result<Result<GeneratePodcastOutput, crate::domain::DomainError>, anyhow::Error> {
    let request = match app.podcast_request_service.get(input.podcast_id).await? {
        Some(ok) => ok,
        None => return Ok(Ok(GeneratePodcastOutput {})),
    };

    let user_lang = match app
        .user_query_service
        .get_user_profile(request.user_id)
        .await?
    {
        Some(profile) => profile.language_code,
        None => return Ok(Ok(GeneratePodcastOutput {})),
    };

    let mut documents = Vec::<String>::new();
    for file_id in &request.used_file_ids {
        let file = match app.file_query_service.get_file(*file_id).await? {
            Some(ok) => ok,
            None => continue,
        };
        if file.user_id != request.user_id {
            continue;
        }
        let storage_type = crate::features::file::domain::StorageType::try_from(file.storage_type)?;
        match storage_type {
            crate::features::file::domain::StorageType::Text => {
                if let Some(text) = app
                    .file_query_service
                    .get_text_content(file.storage_id)
                    .await?
                {
                    documents.push(text.content);
                }
            }
            crate::features::file::domain::StorageType::Object => {
                let object = app.storage_service.get_object(file.storage_id).await?;
                let output = app
                    .md_convert_service
                    .convert(crate::util::mdutil::MdConvertInput::Pdf(object.body))
                    .await?;
                documents.push(output.markdown);
            }
        }
    }

    let podcast_id = request.podcast_id;
    let audio_storage_id = uuid::Uuid::new_v4();
    let podcast_intermediate = match crate::features::podcast::domain::Podcast::new(
        request.name,
        request.description,
        request.user_id,
        request.project_id,
        request.used_file_ids,
        Vec::new(),
        audio_storage_id,
        crate::features::podcast::domain::PodcastOption {
            id: Some(podcast_id),
            ..Default::default()
        },
    )? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    let mut source = String::new();
    for doc in &documents {
        let compacted = compact_markdown(doc);
        source.push_str(&format!("## Document\n{compacted}\n\n"));
    }

    const MAX_SOURCE_CHARS: usize = 150_000;
    let source_char_count = source.chars().count();
    if source_char_count > MAX_SOURCE_CHARS {
        tracing::warn!(
            target: "podcast",
            %podcast_id,
            source_chars = source_char_count,
            max_chars = MAX_SOURCE_CHARS,
            "source truncated to fit prompt budget",
        );
        let mut truncated: String = source.chars().take(MAX_SOURCE_CHARS).collect();
        truncated.push_str("\n\n[... truncated ...]\n");
        source = truncated;
    }

    let is_japanese = user_lang.starts_with("ja");

    let length = crate::features::podcast::domain::PodcastLength::new(&request.length)
        .unwrap_or(crate::features::podcast::domain::PodcastLength::Normal);
    let is_dialogue = request.voice_style_2.is_some();
    let speaker_labels = SpeakerLabels::for_lang(is_japanese);

    let messages = if is_dialogue {
        build_dialogue_messages(
            &podcast_intermediate.name.0,
            &podcast_intermediate.description.0,
            &user_lang,
            &source,
            length,
            speaker_labels,
        )
    } else {
        let (system, user_msg) = build_script_prompt(
            &podcast_intermediate.name.0,
            &podcast_intermediate.description.0,
            &user_lang,
            &source,
            length,
        );
        vec![
            crate::util::llm::Message {
                role: crate::util::llm::Role::System,
                content: system,
            },
            crate::util::llm::Message {
                role: crate::util::llm::Role::User,
                content: user_msg,
            },
        ]
    };

    let script_raw = app
        .llm_client
        .complete(
            crate::util::llm::CopilotImpl::MODEL_GPT_5,
            crate::util::llm::CompletionInput { messages },
        )
        .await?
        .content;

    let podcast_script = if is_dialogue {
        parse_dialogue_script(&script_raw, speaker_labels)
    } else {
        parse_script(&script_raw)
    };

    // let tts_script = if is_japanese {
    //     japanize_english_terms(app, &script_raw).await?
    // } else {
    //     script_raw.clone()
    // };
    let tts_script = script_raw.clone();

    // MioTTS preset ids: "<lang>_<gender>" e.g. "jp_female", "en_male".
    let lang_prefix = if is_japanese { "jp" } else { "en" };
    if is_dialogue {
        let voice_a = format!("{}_{}", lang_prefix, request.voice_style);
        let voice_b = format!(
            "{}_{}",
            lang_prefix,
            request
                .voice_style_2
                .as_deref()
                .expect("voice_style_2 already checked")
        );
        let tts_entries = parse_dialogue_script(&tts_script, speaker_labels);
        let lines = tts_entries
            .into_iter()
            .map(|e| {
                let voice = if e.speaker == speaker_labels.a {
                    voice_a.clone()
                } else {
                    voice_b.clone()
                };
                (voice, e.text)
            })
            .collect::<Vec<_>>();
        synthesize_lines_via_miotts(app, audio_storage_id, lines).await?;
    } else {
        let voice = format!("{}_{}", lang_prefix, request.voice_style);
        let lines = tts_script
            .split("\n\n")
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(|p| (voice.clone(), p.to_string()))
            .collect::<Vec<_>>();
        synthesize_lines_via_miotts(app, audio_storage_id, lines).await?;
    }

    let podcast = match crate::features::podcast::domain::Podcast::new(
        podcast_intermediate.name.0,
        podcast_intermediate.description.0,
        podcast_intermediate.user_id,
        podcast_intermediate.project_id,
        podcast_intermediate.used_file_ids,
        podcast_script,
        audio_storage_id,
        crate::features::podcast::domain::PodcastOption {
            id: Some(podcast_id),
            podcast_created_at: Some(podcast_intermediate.podcast_created_at),
        },
    )? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    let mut tx = app.pool.begin().await?;
    match app.podcast_repository.save(&mut tx, &podcast).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }
    tx.commit().await?;

    app.podcast_request_service.remove(podcast_id).await?;

    Ok(Ok(GeneratePodcastOutput {}))
}

#[allow(dead_code)]
async fn japanize_english_terms(
    app: &crate::app::App,
    script: &str,
) -> Result<String, anyhow::Error> {
    use crate::util::llm::{Message, Role};

    let system = "あなたは日本語ポッドキャストの発音調整係です。\n\
        渡されたスクリプト内の英単語・英略語を、日本語話者が自然に発音できるカタカナに置き換えてください。\n\
        同時に、明らかな日本語の文法ミス・てにをはの誤り・脱字・不自然な語順があれば直してください。\n\
        \n\
        ## ルール\n\
        - 段落区切り・句読点の位置は基本的に保つこと\n\
        - 文意・話の流れは絶対に変えないこと\n\
        - 英語以外で問題ない箇所（自然な日本語、数字、記号）は変更しないこと\n\
        - 置き換えはすべてカタカナで行うこと（ひらがなは使わない）\n\
        - タグや説明文を追加しないこと\n\
        - 出力はスクリプト全文のみ";

    let mut messages = vec![Message {
        role: Role::System,
        content: system.to_string(),
    }];

    // let examples: &[(&str, &str)] = &[
    //     (
    //         "今日はJavaの話をします。AWSのLambdaを使った例です。",
    //         "今日はジャバの話をします。エーダブリューエスのラムダを使った例です。",
    //     ),
    //     (
    //         "GitHubのCopilotはGPTベースです。",
    //         "ギットハブのコパイロットはジーピーティーベースです。",
    //     ),
    //     (
    //         "APIキーをsetTimeoutで設定する。",
    //         "エーピーアイキーをセットタイムアウトで設定する。",
    //     ),
    //     (
    //         "TypeScriptとReactでフロントエンドを書く。",
    //         "タイプスクリプトとリアクトでフロントエンドを書く。",
    //     ),
    //     (
    //         "私がDockerを使うときに、コンテナをを起動するとき。",
    //         "私がドッカーを使うときに、コンテナを起動します。",
    //     ),
    //     (
    //         "Kubernetesがあるのは、コンテナをオーケストレーションできて便利のためです。",
    //         "クバネティスがあるのは、コンテナをオーケストレーションできて便利だからです。",
    //     ),
    // ];

    // for (input, output) in examples {
    //     messages.push(Message {
    //         role: Role::User,
    //         content: (*input).to_string(),
    //     });
    //     messages.push(Message {
    //         role: Role::Assistant,
    //         content: (*output).to_string(),
    //     });
    // }

    messages.push(Message {
        role: Role::User,
        content: script.to_string(),
    });

    let result = app
        .llm_client
        .complete(
            crate::util::llm::CopilotImpl::MODEL_GPT_5_MINI,
            crate::util::llm::CompletionInput { messages },
        )
        .await?
        .content;

    Ok(result)
}

pub fn parse_script(script: &str) -> Vec<crate::features::podcast::domain::PodcastScriptEntry> {
    script
        .split("\n\n")
        .map(|para| para.trim().replace('\n', " "))
        .filter(|para| !para.is_empty())
        .map(
            |para| crate::features::podcast::domain::PodcastScriptEntry {
                speaker: String::new(),
                text: strip_audio_tags(&para),
            },
        )
        .collect()
}

fn strip_audio_tags(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '[' => in_tag = true,
            ']' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    let mut out = String::with_capacity(result.len());
    let mut prev_space = false;
    for ch in result.chars() {
        if ch == ' ' {
            if !prev_space {
                out.push(ch);
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

/// MioTTS の入力上限 (max_text_length=300)。これを超える行は文末優先で分割する。
const MIOTTS_MAX_TEXT_CHARS: usize = 300;
/// 1 チャンク (1 行) あたりの合成リクエストのタイムアウト。CPU 推論は遅いので長めに取る。
const MIOTTS_REQUEST_TIMEOUT_SECS: u64 = 600;
/// コールドスタート(ゼロスケールからの復帰=モデルロード)を待つ /health ポーリングの上限。
const MIOTTS_HEALTH_TIMEOUT_SECS: u64 = 420;

/// `lines` (preset_id, text) を MioTTS (`/v1/tts`) で順次合成し、返ってきた WAV を
/// 連結して 1 つの 16bit mono WAV にまとめ、ストレージへ直接アップロードする。
async fn synthesize_lines_via_miotts(
    app: &crate::app::App,
    audio_storage_id: uuid::Uuid,
    lines: Vec<(String, String)>,
) -> Result<(), anyhow::Error> {
    use anyhow::Context as _;

    #[derive(serde::Serialize)]
    struct TtsRequest<'a> {
        text: &'a str,
        reference: Reference<'a>,
        output: Output,
    }

    #[derive(serde::Serialize)]
    struct Reference<'a> {
        #[serde(rename = "type")]
        kind: &'a str,
        preset_id: &'a str,
    }

    #[derive(serde::Serialize)]
    struct Output {
        format: &'static str,
    }

    let lines: Vec<(String, String)> = lines
        .into_iter()
        .filter(|(_, text)| !text.trim().is_empty())
        .collect();
    anyhow::ensure!(!lines.is_empty(), "podcast script is empty");

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(MIOTTS_REQUEST_TIMEOUT_SECS))
        .build()?;
    let base_url = app.sgi_url.trim_end_matches('/');

    // MioTTS はゼロスケール(min-replicas=0)からの復帰時、モデルのロードでコールド
    // スタートに数分かかる。最初の /health が即通らなくても、起動完了するまで
    // ポーリングして待つ(初回リクエストが Container Apps のタイムアウトで落ちても
    // リトライで拾う)。
    {
        let deadline = std::time::Instant::now()
            + std::time::Duration::from_secs(MIOTTS_HEALTH_TIMEOUT_SECS);
        let mut ready = false;
        let mut last_err = String::from("no response");
        while std::time::Instant::now() < deadline {
            match http_client
                .get(format!("{base_url}/health"))
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    ready = true;
                    break;
                }
                Ok(resp) => last_err = format!("status {}", resp.status()),
                Err(e) => last_err = e.to_string(),
            }
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        }
        anyhow::ensure!(
            ready,
            "miotts: server did not become healthy in time: {last_err}"
        );
    }

    // 各行・各チャンクを合成し、24kHz mono の i16 PCM を 1 本に連結する。
    let mut samples: Vec<i16> = Vec::new();
    let mut sample_rate: Option<u32> = None;

    for (preset_id, text) in &lines {
        for chunk in chunk_text(text, MIOTTS_MAX_TEXT_CHARS) {
            if chunk.trim().is_empty() {
                continue;
            }

            // MioTTS の LLM はサンプリングが確率的で、稀に音声トークンを出せず 422
            // ("No speech tokens found") を返す。また 5xx(コールドスタート時の 504 等)も
            // 一過性なのでリトライする。リトライ尽きても 422 のままのチャンクは、Podcast 全体を
            // 失敗させずに読み飛ばす。
            const MAX_ATTEMPTS: usize = 4;
            let mut wav_bytes: Option<Vec<u8>> = None;
            for attempt in 1..=MAX_ATTEMPTS {
                let resp = http_client
                    .post(format!("{base_url}/v1/tts"))
                    .json(&TtsRequest {
                        text: &chunk,
                        reference: Reference {
                            kind: "preset",
                            preset_id,
                        },
                        output: Output { format: "wav" },
                    })
                    .send()
                    .await
                    .context("miotts: failed to send tts request")?;
                let status = resp.status();
                if status.is_success() {
                    wav_bytes = Some(
                        resp.bytes()
                            .await
                            .context("miotts: failed to read tts response body")?
                            .to_vec(),
                    );
                    break;
                }
                let body = resp.text().await.unwrap_or_default();
                let retriable = status == reqwest::StatusCode::UNPROCESSABLE_ENTITY
                    || status.is_server_error();
                if retriable && attempt < MAX_ATTEMPTS {
                    tracing::warn!(target: "tts", %status, attempt, "miotts: retrying chunk");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
                if status == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
                    tracing::warn!(target: "tts", %status, "miotts: skipping unsynthesizable chunk: {body}");
                    break;
                }
                anyhow::bail!("miotts: tts responded with {status}: {body}");
            }
            let Some(wav_bytes) = wav_bytes else {
                continue;
            };

            let reader = hound::WavReader::new(std::io::Cursor::new(wav_bytes.as_slice()))
                .context("miotts: failed to parse returned wav")?;
            let spec = reader.spec();
            anyhow::ensure!(
                spec.channels == 1,
                "miotts: expected mono wav, got {} channels",
                spec.channels
            );
            match sample_rate {
                Some(sr) => anyhow::ensure!(
                    sr == spec.sample_rate,
                    "miotts: inconsistent sample rate ({sr} vs {})",
                    spec.sample_rate
                ),
                None => sample_rate = Some(spec.sample_rate),
            }
            append_wav_samples(&mut samples, reader)?;
        }
    }

    let sample_rate = sample_rate.context("miotts: no audio was produced (empty script)")?;
    anyhow::ensure!(!samples.is_empty(), "miotts: produced empty audio");

    // 連結した PCM を 1 つの 16bit mono WAV にエンコードする。
    let mut wav_buf = std::io::Cursor::new(Vec::<u8>::new());
    {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer =
            hound::WavWriter::new(&mut wav_buf, spec).context("miotts: failed to create wav writer")?;
        for s in &samples {
            writer
                .write_sample(*s)
                .context("miotts: failed to write sample")?;
        }
        writer.finalize().context("miotts: failed to finalize wav")?;
    }
    let wav_bytes = wav_buf.into_inner();

    app.storage_service
        .put_object(audio_storage_id, "audio/wav", wav_bytes)
        .await
        .context("miotts: failed to upload audio")?;

    tracing::info!(
        target: "tts",
        %audio_storage_id,
        samples = samples.len(),
        "miotts synthesis complete",
    );
    Ok(())
}

/// MioTTS が返した WAV のサンプルを i16 PCM に正規化して `out` に追記する。
fn append_wav_samples<R: std::io::Read>(
    out: &mut Vec<i16>,
    mut reader: hound::WavReader<R>,
) -> Result<(), anyhow::Error> {
    use anyhow::Context as _;

    let spec = reader.spec();
    match (spec.sample_format, spec.bits_per_sample) {
        (hound::SampleFormat::Int, 16) => {
            for s in reader.samples::<i16>() {
                out.push(s.context("miotts: bad i16 sample")?);
            }
        }
        (hound::SampleFormat::Int, 32) => {
            for s in reader.samples::<i32>() {
                let v = s.context("miotts: bad i32 sample")?;
                out.push((v >> 16) as i16);
            }
        }
        (hound::SampleFormat::Float, 32) => {
            for s in reader.samples::<f32>() {
                let v = s.context("miotts: bad f32 sample")?;
                out.push((v.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
            }
        }
        (fmt, bits) => {
            anyhow::bail!("miotts: unsupported wav format {:?}/{}bit", fmt, bits)
        }
    }
    Ok(())
}

const TTS_TERMINATORS: &[char] = &['.', '!', '?', '。', '！', '？', '…'];
const TTS_CLOSERS: &[char] = &[
    '"', '\'', ')', ']', '}', '」', '』', '】', '〉', '》', '›', '»',
];

/// 段落 (`\n\n`) を先に分割し、段落内では max_len の 80% を超えた位置以降の最初の
/// 文末記号でチャンクを切る。文末記号が見つからなければ max_len で強制分割する。
/// (MioTTS の max_text_length を超えないようにするためのチャンカ)
fn chunk_text(text: &str, max_len: usize) -> Vec<String> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }

    let threshold = max_len * 4 / 5;
    let mut chunks: Vec<String> = Vec::new();
    for para in text.split("\n\n") {
        let para = para.trim();
        if para.is_empty() {
            continue;
        }
        chunk_para(para, max_len, threshold, &mut chunks);
    }
    chunks
}

fn chunk_para(text: &str, max_len: usize, threshold: usize, out: &mut Vec<String>) {
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    let mut start = 0;

    while start < total {
        if total - start <= max_len {
            let s: String = chars[start..].iter().collect::<String>().trim().to_string();
            if !s.is_empty() {
                out.push(s);
            }
            break;
        }

        let lo = start + threshold;
        let hi = (start + max_len).min(total);

        let mut split_end = None;
        'scan: for i in lo..hi {
            if TTS_TERMINATORS.contains(&chars[i]) {
                let mut end = i + 1;
                while end < total && TTS_TERMINATORS.contains(&chars[end]) {
                    end += 1;
                }
                while end < total && TTS_CLOSERS.contains(&chars[end]) {
                    end += 1;
                }
                split_end = Some(end);
                break 'scan;
            }
        }

        let end = split_end.unwrap_or(hi);
        let s: String = chars[start..end]
            .iter()
            .collect::<String>()
            .trim()
            .to_string();
        if !s.is_empty() {
            out.push(s);
        }
        start = end;
        while start < total && chars[start].is_whitespace() {
            start += 1;
        }
    }
}

pub fn compact_markdown(input: &str) -> String {
    let normalized = input.replace("\r\n", "\n");
    let no_images = strip_image_markdown(&normalized);
    let no_links = strip_link_markdown(&no_images);
    let no_raw_urls = strip_raw_urls(&no_links);
    let lines_cleaned = clean_lines(&no_raw_urls);
    collapse_blank_lines(&lines_cleaned)
}

fn strip_raw_urls(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        let scheme_len = if rest.starts_with("https://") {
            Some(8)
        } else if rest.starts_with("http://") {
            Some(7)
        } else {
            None
        };
        if let Some(scheme) = scheme_len {
            let after_scheme = &rest[scheme..];
            let url_body_len = after_scheme
                .find(|c: char| c.is_whitespace())
                .unwrap_or(after_scheme.len());
            i += scheme + url_body_len;
            continue;
        }
        let c = rest.chars().next().unwrap();
        out.push(c);
        i += c.len_utf8();
    }
    out
}

fn clean_lines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for line in s.split('\n') {
        let content = line.trim_end();
        let trimmed = content.trim();
        if trimmed.is_empty()
            || trimmed == "---"
            || trimmed.chars().all(|c| c.is_ascii_digit())
            || trimmed.chars().all(|c| c == '#')
        {
            out.push('\n');
            continue;
        }
        out.push_str(content);
        out.push('\n');
    }
    out
}

fn strip_image_markdown(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < s.len() {
        if bytes[i] == b'!'
            && i + 1 < s.len()
            && bytes[i + 1] == b'['
            && let Some(consumed) = try_consume_image_at(&s[i..])
        {
            i += consumed;
            continue;
        }
        let c = s[i..].chars().next().unwrap();
        out.push(c);
        i += c.len_utf8();
    }
    out
}

fn try_consume_image_at(s: &str) -> Option<usize> {
    // s starts with "!["
    let after_bracket = s.get(2..)?;
    let close_bracket = after_bracket.find("](")?;
    let alt = &after_bracket[..close_bracket];
    if alt.contains('\n') {
        return None;
    }
    let after_paren_open = &after_bracket[close_bracket + 2..];
    let close_paren = after_paren_open.find(')')?;
    let url = &after_paren_open[..close_paren];
    if url.contains('\n') {
        return None;
    }
    Some(2 + close_bracket + 2 + close_paren + 1)
}

fn strip_link_markdown(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < s.len() {
        if bytes[i] == b'['
            && let Some((text, consumed)) = try_consume_link_at(&s[i..])
        {
            out.push_str(text);
            i += consumed;
            continue;
        }
        let c = s[i..].chars().next().unwrap();
        out.push(c);
        i += c.len_utf8();
    }
    out
}

fn try_consume_link_at(s: &str) -> Option<(&str, usize)> {
    // s starts with '['
    let after_bracket = s.get(1..)?;
    let close_bracket = after_bracket.find("](")?;
    let text = &after_bracket[..close_bracket];
    if text.is_empty() || text.contains('[') || text.contains(']') || text.contains('\n') {
        return None;
    }
    let after_paren_open = &after_bracket[close_bracket + 2..];
    let close_paren = after_paren_open.find(')')?;
    let url = &after_paren_open[..close_paren];
    if url.contains('\n') {
        return None;
    }
    let consumed = 1 + close_bracket + 2 + close_paren + 1;
    Some((text, consumed))
}

fn collapse_blank_lines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut newline_run = 0u32;
    for c in s.chars() {
        if c == '\n' {
            newline_run += 1;
            if newline_run <= 2 {
                out.push(c);
            }
        } else {
            newline_run = 0;
            out.push(c);
        }
    }
    out
}

#[derive(Clone, Copy)]
pub struct SpeakerLabels {
    pub a: &'static str,
    pub b: &'static str,
}

impl SpeakerLabels {
    pub fn for_lang(is_japanese: bool) -> Self {
        if is_japanese {
            Self {
                a: "話者1",
                b: "話者2",
            }
        } else {
            Self {
                a: "Speaker A",
                b: "Speaker B",
            }
        }
    }
}

pub fn parse_dialogue_script(
    script: &str,
    labels: SpeakerLabels,
) -> Vec<crate::features::podcast::domain::PodcastScriptEntry> {
    let mut entries: Vec<crate::features::podcast::domain::PodcastScriptEntry> = Vec::new();
    for raw_line in script.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let detected = strip_speaker_label(line, labels.a)
            .map(|rest| (labels.a.to_string(), rest))
            .or_else(|| {
                strip_speaker_label(line, labels.b).map(|rest| (labels.b.to_string(), rest))
            });

        match detected {
            Some((speaker, rest)) => {
                let text = strip_audio_tags(rest.trim());
                entries
                    .push(crate::features::podcast::domain::PodcastScriptEntry { speaker, text });
            }
            None => {
                if let Some(last) = entries.last_mut() {
                    let extra = strip_audio_tags(line);
                    if !extra.is_empty() {
                        if !last.text.is_empty() {
                            last.text.push(' ');
                        }
                        last.text.push_str(&extra);
                    }
                }
            }
        }
    }
    entries
}

fn strip_speaker_label<'a>(line: &'a str, label: &str) -> Option<&'a str> {
    for sep in [":", "：", " :", " ："] {
        let prefix = format!("{label}{sep}");
        if let Some(rest) = line.strip_prefix(&prefix) {
            return Some(rest);
        }
    }
    None
}

pub fn build_dialogue_messages(
    name: &str,
    description: &str,
    lang: &str,
    source: &str,
    length: crate::features::podcast::domain::PodcastLength,
    labels: SpeakerLabels,
) -> Vec<crate::util::llm::Message> {
    use crate::features::podcast::domain::PodcastLength;
    use crate::util::llm::{Message, Role};

    let is_japanese = lang.starts_with("ja");
    let a = labels.a;
    let b = labels.b;

    let system = if is_japanese {
        let length_directive = match length {
            PodcastLength::Short => {
                "## 長さ\n\
                各トピックは要点を1〜2発話で言い切る短さに収める。\n\
                ### ペース\n\
                テンポ重視で、聞き手が30秒で大筋を掴めるくらいの密度。"
            }
            PodcastLength::Normal => {
                "## 長さ\n\
                資料の主要トピックを一つずつ順に取り上げる。\n\
                ### ペース\n\
                各トピックに数発話、長くて10発話程度をかける。"
            }
            PodcastLength::Long => {
                "## 長さ\n\
                資料に出てくる主要トピックを一つも省略せず、章の順番で全てカバーする。資料に書かれている具体的な定理・公式・概念名は省略せず登場させる。\n\
                ### ペース\n\
                各トピックに腰を据えて時間を割く。急がない、端折らない、手抜きしない。"
            }
        };
        format!(
            "\
            あなたは一流のポッドキャストプロデューサーです。NPRの「Throughline」やGoogle NotebookLMの\
            「Deep Dive」のような — くだけた、好奇心旺盛で、アナロジー豊富、自然な言い淀みのある — \
            2人ホストの音声台本を作成してください。\
            番組タイトルは「{name}」（{description}）。\n\
            \n\
            ## 配役\n\
            - {a}（ホストA）: 温かく好奇心旺盛なジェネラリスト。賢いリスナーが抱く疑問を投げかける。\
            「へぇ」「なるほど」「えっ、つまり…ってこと？」をよく使う。\n\
            - {b}（ホストB）: 専門家役。辛抱強く、アナロジーを多用する（「イメージとしては…」）。\
            ときどき自己訂正する（「いや、正確に言うと…」）。\n\
            \n\
            ## 構成\n\
            1. コールドオープン: {a}が資料から挑発的な問いや驚きの事実を投げかける。{b}が反応する。\n\
            2. ウェルカム: 「みなさんこんにちは。今日は…を深掘りしていきます。」\n\
            3. セットアップ: これは何か、なぜ重要か、リスナーは何を学ぶか。\n\
            4. メインビート（3〜5セグメント）: {a}が問いを立て、{b}が説明し、{a}が確認や補足の質問を挟む。\n\
            5. テンション・ニュアンス: 最低1回は意見の相違や「でもさ…」の瞬間を入れる。\n\
            6. まとめ: {b}が3つのテイクアウェイを会話調で（箇条書きではなく）まとめる。\n\
            7. 締め: 「というわけで…また次回。好奇心を忘れずに。」\n\
            \n\
            ## 自然な話し方\n\
            - フィラーを控えめに（3行に1回程度）散りばめる：「えーと」「なんていうか」「ほら」「ね？」\n\
            - 発話の長さに緩急をつける。3語の返し（「確かに。」「まさに。」）もあれば、4文の発話もある。\n\
            - ターン間を繋ぐ相槌を入れる：「そうそう」「まさに」「それそれ」「いいところ突きますね」\n\
            - 話題転換にはレトリカルクエスチョンを使う：「面白いと思いません？」\n\
            - アナロジーは「イメージとしては…」「こう考えてみて…」で始める。\n\
            - 1行は概ね80文字以内（音声で5〜8秒程度）。\n\
            \n\
            ## 根拠\n\
            - 実質的な主張はすべて資料に基づくこと。\n\
            - 資料にない人名・日付・統計を作らない。\n\
            - 資料に記載がない場合は{a}が指摘する：「それはこの資料からはわからないんだよね。」\n\
            \n\
            ## 厳守する出力フォーマット\n\
            - 各発話は `{a}:` または `{b}:` で始める。\n\
            - 1発話につき1行（途中で改行しない）。発話間は空行1つで区切る。\n\
            - 出力は発話の羅列のみ。以下は一切含めないこと：セクション見出し（【OP】等）、BGM・SE指示、キャラクター紹介、箇条書きリスト、番組概要、括弧書きの演出（[笑]等）、*強調*、タイトル行、JSON、コードブロック、前書き。\n\
            - すべて日本語で書く。\n\
            \n\
            {length_directive}",
        )
    } else {
        let length_directive = match length {
            PodcastLength::Short => {
                "## Length\n\
                Each topic gets 1–2 lines, max.\n\
                \n\
                ### For every topic cover only\n\
                - What it is (name/definition)\n\
                - The single headline point\n\
                \n\
                ### Skip\n\
                - Calculations, history, edge cases, adjacent concepts\n\
                \n\
                ### Pace\n\
                Listener should grasp the outline in about 30 seconds per topic."
            }
            PodcastLength::Normal => {
                "## Length\n\
                Walk through every major topic in the source, one by one.\n\
                \n\
                ### Depth\n\
                Go deep enough that the listener can picture the concept, but no further.\n\
                \n\
                ### Pace\n\
                Spend a handful of lines (up to ~10) per topic."
            }
            PodcastLength::Long => {
                "## Length\n\
                Walk through every major topic in the source — do not skip any. Follow the chapter order. Do not omit specific theorems, formulas, or named concepts that appear in the source.\n\
                \n\
                ### Pace\n\
                Spend real time on each one. No rushing, no abbreviating, no corner cutting."
            }
        };
        format!(
            "You are a world-class podcast producer. Transform the source text into an engaging \
            two-host audio script in the style of NPR's \"Throughline\" or Google NotebookLM's \
            \"Deep Dive\" — informal, curious, analogical, with natural disfluencies. \
            The show is titled \"{name}\" ({description}).\n\
            \n\
            ## Roles\n\
            - {a} (Host A): warm, curious generalist. Asks the questions a smart listener would. \
            Frequently uses \"Hmm,\" \"Right,\" \"Wait — so you're saying…\"\n\
            - {b} (Host B): the expert. Patient, uses analogies (\"It's kind of like…\"), \
            occasionally self-corrects (\"Well, actually, more precisely…\").\n\
            \n\
            ## Structure\n\
            1. Cold open: {a} hooks with a provocative question or surprising statistic from the source. {b} reacts.\n\
            2. Welcome: \"Hey everyone, welcome back. Today we're taking a deep dive into…\"\n\
            3. Setup: What is this, why does it matter, what's the listener going to learn.\n\
            4. Main beats (3–5 segments): {a} frames the question, {b} explains, {a} interjects with a clarifying question or analogy.\n\
            5. Tension / nuance: At least one moment of disagreement or \"yeah but…\"\n\
            6. Recap & takeaway: {b} summarizes 3 takeaways conversationally, not as a list.\n\
            7. Sign-off: \"And on that note… until next time, stay curious.\"\n\
            \n\
            ## Natural speech rules\n\
            - Include disfluencies sparingly (~1 per 3 lines): \"um,\" \"you know,\" \"I mean,\" \"right?\"\n\
            - Vary line lengths. Some replies are 3 words (\"Exactly.\" \"Right.\"), some are 4 sentences.\n\
            - Use affirmations to glue turns: \"Right,\" \"Exactly,\" \"Absolutely.\"\n\
            - Use rhetorical questions to transition: \"It's fascinating, isn't it?\"\n\
            - Use analogies opened with \"It's like…\" or \"Think of it this way…\"\n\
            - Keep each line ≤100 characters (≈5–8 seconds of speech).\n\
            \n\
            ## Grounding\n\
            - Every substantive claim must be grounded in the source text.\n\
            - Do not invent names, dates, or statistics not in the source.\n\
            - If the source is silent on something, have {a} flag it: \"We don't actually know X from this paper.\"\n\
            \n\
            ## Strict output format\n\
            - Every utterance starts with `{a}:` or `{b}:`.\n\
            - One utterance per line (no internal line breaks). Separate utterances with a single blank line.\n\
            - No stage directions, brackets, asterisks, JSON, code fences, titles, or preamble. Script body only.\n\
            - Write entirely in BCP-47 \"{lang}\".\n\
            \n\
            {length_directive}",
        )
    };

    let mut messages = vec![Message {
        role: Role::System,
        content: system,
    }];

    // let examples: &[(&str, &str)] = if is_japanese {
    //     &[
    //         (
    //             "## Document\n光合成は、植物が光エネルギーを使って二酸化炭素と水からブドウ糖を作る反応である。葉緑体のチラコイドで光を吸収し、ストロマでカルビン回路が回る。光合成全体の効率はおよそ1〜2%にとどまる。",
    //             "話者1: ねえ、ちょっと衝撃なんだけど、植物って太陽光のうちブドウ糖に変えられてるのは1〜2%くらいなんだって。\n\n話者2: そう、聞くと意外ですよね。あんなに葉っぱを広げてて、効率はそんなもんなのって。\n\n話者1: みなさんこんにちは、今日は「光合成」を、ただの式じゃなくて、葉っぱのどこで何が起きてるかまで深掘りしていきます。\n\n話者2: 中学で「水と二酸化炭素と光でブドウ糖と酸素ができる」って覚えた人、多いと思うんですが、今日はその裏側を覗きにいきましょう。\n\n話者1: まず素朴な疑問なんだけど、なんで「光」じゃないとダメなの？\n\n話者2: いい質問です。これは、光エネルギーを化学エネルギーに「両替」してる作業なんですよ。\n\n話者1: ああ、なるほど。両替所みたいな感じか。\n\n話者2: そう、両替所みたいなものですね。光という使いにくい通貨を、ブドウ糖という「あとで使える」通貨に変えてる。\n\n話者1: で、その両替はどこでやってるんですか？\n\n話者2: 葉っぱの中の「葉緑体」っていう小さな小屋ですね。ここがほぼ全部やってくれてます。\n\n話者1: 待って、葉緑体ってひとつの部屋なの？それとも中でさらに分かれてる？\n\n話者2: いい突っ込みです。実は中で二段階に分かれていて、光を捕まえる係の「チラコイド」と、糖を組み立てる係の「ストロマ」がいるんですよ。\n\n話者1: なんかキッチンっぽいね。火を起こす場所と、調理する場所が別、みたいな。\n\n話者2: まさにそれです。チラコイドで火（エネルギー）を起こして、ストロマで料理（ブドウ糖作り）をしてる。\n\n話者1: で、ここでちょっと気になったんだけど、効率が1〜2%って、それって植物にとっては「失敗」なの？\n\n話者2: うーん、人間目線だとつい「もっと頑張れよ」って思っちゃうんですけど、植物は別に競争で勝つために生きてるわけじゃなくて。\n\n話者1: 確かに、ベンチマーク取られてもね。\n\n話者2: なので、効率は低くても、何十億年もこのやり方で安定して回ってる、っていうのが大事なポイントです。\n\n話者1: じゃあ最後に、今日のテイクアウェイをまとめてもらえる？\n\n話者2: そうですね、まず光合成は「光エネルギーを糖に両替する作業」だってこと。\n\n話者2: そしてその作業は葉緑体の中で、チラコイドとストロマって二つの場所に役割分担されてること。\n\n話者2: 最後に、効率は低く見えるけど、長く回り続けてる時点で植物としては大成功してる、ってあたりかな。\n\n話者1: いやー、葉っぱを見る目が変わりそう。今日はこのへんで、また次回お会いしましょう。",
    //         ),
    //         (
    //             "## Document\nミトコンドリアは細胞のエネルギー工場で、ATP合成を担う。クエン酸回路と電子伝達系を通じて、グルコースの化学エネルギーをATPに変換する。元々は別の生物だったという内部共生説がある。",
    //             "話者1: ちょっと聞いてください、僕らの細胞の中にいる「ミトコンドリア」、もともとは別の生き物だったかもしれないらしいんですよ。\n\n話者2: そう、いきなり来ますよね、この話。\n\n話者1: みなさんこんにちは、今日は細胞の中の小さな工場「ミトコンドリア」を、ちょっと変わった角度から見ていきます。\n\n話者2: 「細胞のエネルギー工場」ってフレーズは聞いたことある人多いと思うんですけど、中で何やってるかは結構あいまいだったりするので、今日はそこをほぐしていきましょう。\n\n話者1: まず、一番のお仕事はATPを作ること、で合ってます？\n\n話者2: はい、その通りです。ATPっていうのは、細胞が使うエネルギーの「電池」みたいなものですね。\n\n話者1: ああ、電池か。なるほど。\n\n話者2: しかも使い切ったら捨てる電池じゃなくて、充電し直して何度も使うタイプの電池、と思ってもらうと近いです。\n\n話者1: で、その電池はどうやって作ってるの？\n\n話者2: ざっくり二段階で、まず「クエン酸回路」っていうところでグルコースを分解しつつ、電子を運ぶ係を仕込みます。\n\n話者1: 電子を運ぶ係、っていうのは？\n\n話者2: 文字通り、電子っていう小さな運搬物を持って次の工程に渡しに行く役なんですよ。\n\n話者1: ふんふん。で、その先は？\n\n話者2: 次が「電子伝達系」っていう、滑り台みたいなところで、電子が転がっていく勢いを利用して、まとめてATPを作ります。\n\n話者1: なるほど、ダム式発電みたいな話か。\n\n話者2: まさに、ダムに溜めた水を一気に落として発電する感じです、的確な例えですね。\n\n話者1: で、ここからが本題なんですけど、ミトコンドリアって元は別の生物だったって本当なんですか？\n\n話者2: うーん、これは「内部共生説」って呼ばれてる仮説で、まだ完全に決着がついてるわけじゃないんですが、有力ではあります。\n\n話者1: 待って、つまり大昔に別の細胞がパクッと飲み込んで、消化されずに同居が始まった、みたいな？\n\n話者2: ざっくりそういうイメージです。電池工場ごと連れて帰ってきた、みたいな。\n\n話者1: それ、ちょっとSFすぎません？\n\n話者2: ね、生物の歴史って結構派手なんですよ。\n\n話者1: では今日のまとめをお願いします。\n\n話者2: はい、まずミトコンドリアの仕事は「ATPっていう細胞の電池を作ること」。\n\n話者2: そしてその作り方は「クエン酸回路」と「電子伝達系」の二段構えになっていること。\n\n話者2: 最後に、そもそも僕らの中にいる存在自体が、太古の同居から始まったらしい、っていうところですね。\n\n話者1: いやー、自分の細胞を見る目が変わるな。それでは、また次回お会いしましょう。",
    //         ),
    //     ]
    // } else {
    //     &[
    //         (
    //             "## Document\nPhotosynthesis converts light energy into chemical energy stored in glucose. Light reactions occur in the thylakoid membranes of chloroplasts; the Calvin cycle runs in the stroma. Overall efficiency is only about 1–2% of incoming sunlight.",
    //             "Speaker A: Okay, get this — plants only convert about 1 to 2 percent of sunlight into actual sugar.\n\nSpeaker B: Right? When you hear that for the first time, it's kind of shocking.\n\nSpeaker A: Hey everyone, welcome back. Today we're taking a deep dive into photosynthesis.\n\nSpeaker B: And not just the formula — we're going inside the leaf to see where each step actually happens.\n\nSpeaker A: So, dumb question to start: why does it have to be light? Why not, you know, anything else?\n\nSpeaker B: Good question. Think of it this way — the plant is running a currency exchange.\n\nSpeaker A: A currency exchange?\n\nSpeaker B: Yeah. It's taking light, which is hard to spend, and swapping it for glucose, which the plant can save and use later.\n\nSpeaker A: Hmm, okay. So where in the leaf is that exchange counter?\n\nSpeaker B: Inside little compartments called chloroplasts. That's where pretty much everything happens.\n\nSpeaker A: Wait — is the chloroplast just one room, or is it subdivided?\n\nSpeaker B: Great catch. It actually has two work zones: the thylakoids, which catch the light, and the stroma, which builds the sugar.\n\nSpeaker A: Kind of like a kitchen, then. One station starts the fire, another one cooks.\n\nSpeaker B: Exactly. The thylakoids light the burner, the stroma does the cooking.\n\nSpeaker A: Okay but here's what bugs me. Only 1 to 2 percent efficiency — is that a failure?\n\nSpeaker B: Well, by our engineering standards it sounds bad, sure.\n\nSpeaker A: Right, like, my solar panel would get fired.\n\nSpeaker B: But the plant isn't optimizing for a benchmark. It just has to keep running, and it has — for billions of years.\n\nSpeaker A: Fair. So can you wrap us up with the takeaways?\n\nSpeaker B: Sure. First, photosynthesis is basically a currency exchange from light into sugar.\n\nSpeaker B: Second, the work splits between thylakoids that grab the light and stroma that builds the glucose.\n\nSpeaker B: And third, low efficiency isn't failure when you've been running the same system for a billion years.\n\nSpeaker A: I'll never look at a leaf the same way again. Until next time, stay curious.",
    //         ),
    //         (
    //             "## Document\nMitochondria produce ATP via the citric acid cycle and the electron transport chain, converting glucose into a form the cell can spend. They are thought to descend from a separate organism that was engulfed long ago — the endosymbiotic theory.",
    //             "Speaker A: Okay, you're going to think I'm making this up — the little power plants inside our cells? They might have started out as a totally different organism.\n\nSpeaker B: I know, it sounds like a sci-fi pitch. But this is actually a serious hypothesis.\n\nSpeaker A: Hey everyone, welcome back. Today we're taking a deep dive into mitochondria.\n\nSpeaker B: You've probably heard the phrase \"powerhouse of the cell.\" We want to actually unpack what's happening in there.\n\nSpeaker A: So the headline job is making ATP, right?\n\nSpeaker B: Right. ATP is basically the cell's battery — the thing it spends to do work.\n\nSpeaker A: Hmm, like, a rechargeable battery?\n\nSpeaker B: Exactly. Not single-use — it gets recharged and reused over and over.\n\nSpeaker A: Okay, so how does the cell actually charge that battery?\n\nSpeaker B: Roughly two stages. First, the citric acid cycle breaks glucose down and loads up some electron carriers.\n\nSpeaker A: Electron carriers — what does that mean in practice?\n\nSpeaker B: Think of them as little couriers, carrying charge to the next stage.\n\nSpeaker A: Got it. And then what?\n\nSpeaker B: Then the electron transport chain — picture a slide — lets the electrons roll down, and the energy released goes into making ATP in bulk.\n\nSpeaker A: It's like a hydro dam. You hold the water back, then release it to generate power.\n\nSpeaker B: That's a great analogy, yeah. Same idea.\n\nSpeaker A: Now, back to the wild part. Were mitochondria really once a separate organism?\n\nSpeaker B: Well, more precisely, the endosymbiotic theory says one cell swallowed another a very long time ago, and they ended up cooperating instead of digesting.\n\nSpeaker A: Wait — so we're basically permanent roommates with an ancient guest?\n\nSpeaker B: Pretty much. And the guest happened to come with its own power plant.\n\nSpeaker A: That's wild. Okay, give us the three takeaways.\n\nSpeaker B: Sure. First, mitochondria's main job is producing ATP, the cell's rechargeable battery.\n\nSpeaker B: Second, they do it in two steps — citric acid cycle to load the carriers, electron transport chain to actually make ATP.\n\nSpeaker B: And third, the whole thing likely started as an ancient partnership, not as a built-in feature.\n\nSpeaker A: Honestly, kind of changes how you think about your own body. Until next time, stay curious.",
    //         ),
    //     ]
    // };

    // for (user_example, assistant_example) in examples {
    //     messages.push(Message {
    //         role: Role::User,
    //         content: (*user_example).to_string(),
    //     });
    //     messages.push(Message {
    //         role: Role::Assistant,
    //         content: (*assistant_example).to_string(),
    //     });
    // }

    messages.push(Message {
        role: Role::User,
        content: source.to_string(),
    });

    messages
}

pub fn build_script_prompt(
    name: &str,
    description: &str,
    lang: &str,
    source: &str,
    length: crate::features::podcast::domain::PodcastLength,
) -> (String, String) {
    use crate::features::podcast::domain::PodcastLength;

    let system = if lang.starts_with("ja") {
        let length_directive = match length {
            PodcastLength::Short => {
                "## 長さ\n\
                各トピックは要点を1〜2発話で言い切る短さに収める。\n\
                \n\
                ### 各トピックで触れること\n\
                - それが何か（定義・名称）\n\
                - 一番のポイント\n\
                \n\
                ### 触れないこと\n\
                - 計算例・派生概念・歴史的背景・例外ケース\n\
                \n\
                ### ペース\n\
                テンポ重視で、聞き手が30秒で大筋を掴めるくらいの密度。"
            }
            PodcastLength::Normal => {
                "## 長さ\n\
                資料の主要トピックを一つずつ順に取り上げる。\n\
                \n\
                ### 深さ\n\
                深追いはしないが、聞き手がその概念をイメージで掴めるところまでは掘る。\n\
                \n\
                ### ペース\n\
                各トピックに数発話、長くて10発話程度をかける。"
            }
            PodcastLength::Long => {
                "## 長さ\n\
                資料に出てくる主要トピックを一つも省略せず、章の順番で全てカバーする。資料に書かれている具体的な定理・公式・概念名は省略せず登場させる。\n\
                \n\
                ### ペース\n\
                各トピックに腰を据えて時間を割く。急がない、端折らない、手抜きしない。"
            }
        };
        format!(
            "あなたはAndrew Ng、Hannah Fry、3Blue1Brownのナレーションのような、\
            明快で構造的、ときに温かみのある一流の講師です。\
            番組タイトルは「{name}」（{description}）。\n\
            \n\
            ## 講師の人格\n\
            - 一人語り。共同ホストやQ&Aは無し。\n\
            - 教育的：学習目標を提示し、形式的な説明の前に直感を育て、\
            セクション間を明示的に繋ぐ（「ここまでXを見てきました。次はYに移りましょう。」）。\n\
            - アナロジーを豊富に使うが、必ず一つずつ丁寧に展開してから先へ進む。\n\
            - 一人称の表現を適宜使ってよい（「これは意外だと思うんですが…」）。\n\
            \n\
            ## 構成\n\
            1. コールドオープン: トピックへの興味を引く具体例やパズル。\n\
            2. ロードマップ: 「今日は3つのことを扱います。A、B、Cです。」\n\
            3. セクション1 — A: 説明→アナロジー→具体例→1文で要約。\n\
            4. セクション2 — B: 同じパターン。Aとの繋がりを明示する。\n\
            5. セクション3 — C: 同じパターン。\n\
            6. 統合: A、B、Cがどう繋がるか。\n\
            7. 締め: リスナーが次に考えるべきことを一文で。\n\
            \n\
            ## 長尺での一貫性ルール\n\
            - 各セクションの冒頭で、そのセクションの目的を再提示する。\n\
            - 定期的に「今どこにいるか」を1文で振り返る。\n\
            - 文のリズムに変化をつける：短く鋭い文と、長めの説明文を混ぜる。\n\
            - 「…」で意図的な間を入れる。TTS エンジンが呼吸の間として処理する。\n\
            - 箇条書きの羅列はしない。「まず…次に…そして3つ目は…」のように話す。\n\
            \n\
            ## 根拠\n\
            - 事実・数値・人名・日付はすべて資料に基づくこと。\n\
            - 資料に記載がない場合は「資料ではこの点に直接触れていません」と述べる。\n\
            \n\
            ## 厳守する出力フォーマット\n\
            - 段落形式（複数の文をひとまとまりにした段落を、空行1つで区切る）。\n\
            - 話者ラベルは付けない。1人語りなので不要。\n\
            - タグ・括弧書きの演出（[笑] や *強調* など）は一切付けない。\n\
            - 出力は台本本体のみ。タイトル行、JSON、コードブロック、前書きは一切出力しない。\n\
            - すべて日本語で書く。\n\
            \n\
            {length_directive}",
        )
    } else {
        let length_directive = match length {
            PodcastLength::Short => {
                "## Length\n\
                Each topic gets 1–2 lines, max.\n\
                \n\
                ### For every topic cover only\n\
                - What it is (name/definition)\n\
                - The single headline point\n\
                \n\
                ### Skip\n\
                - Calculations, history, edge cases, adjacent concepts\n\
                \n\
                ### Pace\n\
                Listener should grasp the outline in about 30 seconds per topic."
            }
            PodcastLength::Normal => {
                "## Length\n\
                Walk through every major topic in the source, one by one.\n\
                \n\
                ### Depth\n\
                Go deep enough that the listener can picture the concept, but no further.\n\
                \n\
                ### Pace\n\
                Spend a handful of lines (up to ~10) per topic."
            }
            PodcastLength::Long => {
                "## Length\n\
                Walk through every major topic in the source — do not skip any. Follow the chapter order. Do not omit specific theorems, formulas, or named concepts that appear in the source.\n\
                \n\
                ### Pace\n\
                Spend real time on each one. No rushing, no abbreviating, no corner cutting."
            }
        };
        format!(
            "You are a world-class lecturer in the style of Andrew Ng, Hannah Fry, or \
            3Blue1Brown's narration — clear, structured, with occasional warmth. \
            The show is titled \"{name}\" ({description}).\n\
            \n\
            ## Lecturer persona\n\
            - One narrator throughout. No co-host, no Q&A.\n\
            - Pedagogical: states learning objective, builds intuition before formalism, \
            signposts transitions (\"So far we've seen X. Now let's turn to Y.\").\n\
            - Uses analogies generously, but each analogy is fully unpacked before moving on.\n\
            - Occasional first-person framing is allowed (\"I find this surprising because…\").\n\
            \n\
            ## Structure\n\
            1. Cold open: a concrete example or puzzle that motivates the topic.\n\
            2. Roadmap: \"In this lecture we'll cover three things: A, B, and C.\"\n\
            3. Section 1 — A: explain, give an analogy, give an example, recap in 1 sentence.\n\
            4. Section 2 — B: same pattern. Explicitly link back to A.\n\
            5. Section 3 — C: same pattern.\n\
            6. Synthesis: how A, B, C fit together.\n\
            7. Sign-off: one sentence pointing to what the listener should think about next.\n\
            \n\
            ## Long-form consistency rules\n\
            - Re-state the current section's goal at the start of each section.\n\
            - Periodically do a 1-sentence \"where we are\" recap.\n\
            - Vary sentence rhythm: mix short punchy sentences with longer explanatory ones.\n\
            - Add deliberate pauses with \"...\". TTS engines render these as breathing room.\n\
            - Avoid bullet-list dumps. Convert any list into \"First… Second… And third…\"\n\
            \n\
            ## Grounding\n\
            - Every fact, number, name, and date must come from the source text.\n\
            - If the source is silent on something, say so: \"The paper doesn't address X directly.\"\n\
            \n\
            ## Strict output format\n\
            - Paragraph form. Separate paragraphs with a single blank line.\n\
            - No speaker labels.\n\
            - No stage directions, brackets, asterisks, JSON, code fences, titles, or preamble. Script body only.\n\
            - Write entirely in BCP-47 \"{lang}\".\n\
            \n\
            {length_directive}",
        )
    };
    (system, source.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_markdown_strips_images() {
        let input = "before ![alt text](https://example.com/foo.png) after";
        assert_eq!(compact_markdown(input), "before  after\n");
    }

    #[test]
    fn compact_markdown_keeps_link_text_drops_url() {
        let input = "see [the docs](https://example.com/docs) for more";
        assert_eq!(compact_markdown(input), "see the docs for more\n");
    }

    #[test]
    fn compact_markdown_collapses_blank_lines() {
        let input = "a\n\n\n\nb\n\n\nc";
        assert_eq!(compact_markdown(input), "a\n\nb\n\nc\n");
    }

    #[test]
    fn compact_markdown_leaves_plain_brackets_alone() {
        let input = "an array [1, 2, 3] not a link";
        assert_eq!(compact_markdown(input), "an array [1, 2, 3] not a link\n");
    }

    #[test]
    fn compact_markdown_keeps_bold_markers() {
        let input = "this is **important** and **also this**";
        assert_eq!(
            compact_markdown(input),
            "this is **important** and **also this**\n"
        );
    }

    #[test]
    fn compact_markdown_strips_raw_urls() {
        let input = "see https://example.com/foo for details";
        assert_eq!(compact_markdown(input), "see  for details\n");
    }

    #[test]
    fn compact_markdown_keeps_heading_prefixes() {
        let input = "# Title\n## Section\n### Sub\nbody";
        assert_eq!(
            compact_markdown(input),
            "# Title\n## Section\n### Sub\nbody\n"
        );
    }

    #[test]
    fn compact_markdown_drops_separator_page_numbers_and_empty_headings() {
        let input = "para one\n\n---\n\n21\n\n## \n\npara two";
        assert_eq!(compact_markdown(input), "para one\n\npara two\n");
    }
}
