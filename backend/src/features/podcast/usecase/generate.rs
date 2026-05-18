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
                    .pdf2md_service
                    .convert(crate::util::pdf2md::Pdf2MdInput { pdf: object.body })
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
        source.push_str(&format!("## Document\n{doc}\n\n"));
    }

    let is_japanese = user_lang.starts_with("ja");

    let outline_system = if is_japanese {
        format!(
            "あなたはポッドキャストプロデューサーです。\
            「{name}」（{description}）というエピソードの詳細な構成を作成してください。\
            各セグメントに以下を含めること：タイトル、①日常的な導入フック、②聴衆の素朴な直感、\
            ③専門家による意外な転換、④わかりやすい具体的な例え、⑤聴衆のahaモーメント、\
            ⑥発展的なトピック、プッシュバックポイント1つ。\
            日本語で書くこと。",
            name = podcast_intermediate.name.0,
            description = podcast_intermediate.description.0,
        )
    } else {
        format!(
            "You are a podcast producer. \
            Create a detailed episode outline for a podcast titled \"{name}\" ({description}). \
            For each segment provide: title, (1) everyday hook, (2) naive intuition, \
            (3) expert subversion, (4) one vivid concrete example, (5) listener's aha moment, \
            (6) extension topic, and one planned pushback moment. \
            Write in BCP-47 \"{lang}\".",
            name = podcast_intermediate.name.0,
            description = podcast_intermediate.description.0,
            lang = user_lang,
        )
    };

    let _outline = app
        .llm_client
        .complete(
            crate::util::llm::CopilotImpl::MODEL_GPT_5,
            crate::util::llm::CompletionInput {
                messages: vec![
                    crate::util::llm::Message {
                        role: crate::util::llm::Role::System,
                        content: outline_system,
                    },
                    crate::util::llm::Message {
                        role: crate::util::llm::Role::User,
                        content: source,
                    },
                ],
            },
        )
        .await?
        .content;

    let script_system = if is_japanese {
        format!(
            "あなたはポッドキャストのナレーターです。「{name}」というエピソードの完全なナレーションスクリプトを書いてください。\n\
            \n\
            ## 形式\n\
            - 一人のナレーターによるモノローグ形式\n\
            - 段落ごとに書き、段落間は空行で区切る\n\
            - 各段落または文の先頭に英語の伝達タグを入れる（使いすぎない）\n\
            \n\
            ## 伝達タグの例\n\
            [informative] 今日のテーマは…\n\
            [explanation] つまりこういうことで…\n\
            [building anticipation] ここからが本題で…\n\
            [questioning] では、なぜそうなるのか。\n\
            [insight] ここが重要なポイントで…\n\
            [reminder] 覚えておいてほしいのは…\n\
            \n\
            ## ルール\n\
            - 冒頭で今回のテーマを初見の聴衆にもわかるように簡潔に紹介してから本題に入ること\n\
            - アウトラインの全セグメントを網羅すること\n\
            - 自然で流れるような話し言葉で書くこと\n\
            - コード・数式・技術的な記号は含めないこと\n\
            - すべて日本語で書くこと\n\
            \n\
            ## アウトライン\n\
            {outline}",
            name = podcast_intermediate.name.0,
            outline = _outline,
        )
    } else {
        format!(
            "You are a podcast narrator. Write a complete narration script for an episode titled \"{name}\".\n\
            \n\
            ## Format\n\
            - Single narrator monologue\n\
            - Write in paragraphs separated by blank lines\n\
            - Place English delivery tags at the start of sentences or paragraphs (use sparingly)\n\
            \n\
            ## Delivery tag examples\n\
            [informative] Today's topic is...\n\
            [explanation] In other words...\n\
            [building anticipation] Here's where it gets interesting —\n\
            [questioning] So why does this happen?\n\
            [insight] The key insight here is...\n\
            [reminder] What's important to remember is...\n\
            \n\
            ## Rules\n\
            - Open with a brief intro that newcomers can follow\n\
            - Cover all outline segments\n\
            - Write in natural, flowing spoken language\n\
            - Never include code, formulas, or raw technical syntax\n\
            - Write entirely in BCP-47 \"{lang}\"\n\
            \n\
            ## Outline\n\
            {outline}",
            name = podcast_intermediate.name.0,
            lang = user_lang,
            outline = _outline,
        )
    };

    let script_raw = app
        .llm_client
        .complete(
            crate::util::llm::CopilotImpl::MODEL_GPT_5,
            crate::util::llm::CompletionInput {
                messages: vec![
                    crate::util::llm::Message {
                        role: crate::util::llm::Role::System,
                        content: script_system,
                    },
                    crate::util::llm::Message {
                        role: crate::util::llm::Role::User,
                        content: if is_japanese {
                            "完全なナレーションスクリプトを生成してください。".to_string()
                        } else {
                            "Generate the complete narration script now.".to_string()
                        },
                    },
                ],
            },
        )
        .await?
        .content;

    let podcast_script = parse_script(&script_raw);

    const TTS_CHUNK_CHARS: usize = 2_500;
    const TTS_VOICE: &str = "Iapetus";
    const TTS_AUDIO_PROFILE: &str =
        "A warm and engaging podcast host with natural conversational energy. \
         Shifts tone fluidly between explaining complex ideas with clarity and \
         reacting with genuine curiosity and humour.";
    const TTS_MAX_ATTEMPTS: u32 = 3;
    let mut audio_content_type = String::new();
    let mut audio = Vec::<u8>::new();
    for chunk in split_script_chunks(&script_raw, TTS_CHUNK_CHARS) {
        let mut last_err: Option<anyhow::Error> = None;
        let mut part: Option<crate::util::tts::SynthesizedAudio> = None;
        for attempt in 0..TTS_MAX_ATTEMPTS {
            match app
                .tts_client
                .synthesize_dialogue(crate::util::tts::SynthesizeDialogueInput {
                    script: chunk.clone(),
                    voice: TTS_VOICE.to_string(),
                    audio_profile: TTS_AUDIO_PROFILE.to_string(),
                })
                .await
            {
                Ok(ok) => {
                    part = Some(ok);
                    break;
                }
                Err(e) => {
                    tracing::warn!(attempt, error = %e, "tts chunk attempt failed");
                    last_err = Some(e);
                    if attempt + 1 < TTS_MAX_ATTEMPTS {
                        let backoff_ms = 1000u64 << attempt;
                        tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }
        let part = match part {
            Some(p) => p,
            None => {
                return Err(last_err
                    .unwrap_or_else(|| anyhow::anyhow!("tts failed without producing an error")));
            }
        };
        if audio_content_type.is_empty() {
            audio_content_type = part.content_type;
        }
        audio.extend(part.audio);
    }

    let sample_rate = crate::util::tts::parse_pcm_sample_rate(&audio_content_type);
    let wav = crate::util::tts::wrap_pcm_as_wav(&audio, sample_rate);

    app.storage_service
        .put_object(audio_storage_id, "audio/wav", wav)
        .await?;

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

fn parse_script(script: &str) -> Vec<crate::features::podcast::domain::PodcastScriptEntry> {
    script
        .split("\n\n")
        .map(|para| para.trim().replace('\n', " "))
        .filter(|para| !para.is_empty())
        .map(|para| crate::features::podcast::domain::PodcastScriptEntry {
            speaker: String::new(),
            text: strip_audio_tags(&para),
        })
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

fn split_script_chunks(script: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for line in script.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !current.is_empty() && current.len() + 1 + line.len() > max_chars {
            chunks.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}
