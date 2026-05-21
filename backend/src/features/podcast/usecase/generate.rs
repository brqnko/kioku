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

    let (system, user_msg) = build_script_prompt(
        &podcast_intermediate.name.0,
        &podcast_intermediate.description.0,
        &user_lang,
        &source,
    );

    let script_raw = app
        .llm_client
        .complete(
            crate::util::llm::CopilotImpl::MODEL_GPT_5,
            crate::util::llm::CompletionInput {
                messages: vec![
                    crate::util::llm::Message {
                        role: crate::util::llm::Role::System,
                        content: system,
                    },
                    crate::util::llm::Message {
                        role: crate::util::llm::Role::User,
                        content: user_msg,
                    },
                ],
            },
        )
        .await?
        .content;

    let podcast_script = parse_script(&script_raw);

    let tts_script = if is_japanese {
        japanize_english_terms(app, &script_raw).await?
    } else {
        script_raw.clone()
    };

    let lang_prefix = if is_japanese { "ja" } else { "en" };
    let tts_voice = format!("{}:{}", lang_prefix, request.voice_style);
    let tts_result = app
        .tts_client
        .synthesize_dialogue(crate::util::tts::SynthesizeDialogueInput {
            script: tts_script,
            voice: tts_voice,
        })
        .await?;
    let wav = crate::util::tts::wrap_pcm_as_wav(&tts_result.audio, tts_result.sample_rate);
    let opus = crate::util::audio::wav_to_opus(wav, 32).await?;
    app.storage_service
        .put_object(audio_storage_id, "audio/ogg", opus)
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

    let examples: &[(&str, &str)] = &[
        (
            "今日はJavaの話をします。AWSのLambdaを使った例です。",
            "今日はジャバの話をします。エーダブリューエスのラムダを使った例です。",
        ),
        (
            "GitHubのCopilotはGPTベースです。",
            "ギットハブのコパイロットはジーピーティーベースです。",
        ),
        (
            "APIキーをsetTimeoutで設定する。",
            "エーピーアイキーをセットタイムアウトで設定する。",
        ),
        (
            "TypeScriptとReactでフロントエンドを書く。",
            "タイプスクリプトとリアクトでフロントエンドを書く。",
        ),
        (
            "私がDockerを使うときに、コンテナをを起動するとき。",
            "私がドッカーを使うときに、コンテナを起動します。",
        ),
        (
            "Kubernetesがあるのは、コンテナをオーケストレーションできて便利のためです。",
            "クバネティスがあるのは、コンテナをオーケストレーションできて便利だからです。",
        ),
    ];

    for (input, output) in examples {
        messages.push(Message {
            role: Role::User,
            content: (*input).to_string(),
        });
        messages.push(Message {
            role: Role::Assistant,
            content: (*output).to_string(),
        });
    }

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

pub fn build_script_prompt(
    name: &str,
    description: &str,
    lang: &str,
    source: &str,
) -> (String, String) {
    let system = if lang.starts_with("ja") {
        format!(
            "あなたはポッドキャストのナレーターです。\
            「{name}」（{description}）の完全なナレーションスクリプトを書いてください。\
            一人のナレーターによるモノローグ形式で段落ごとに書き、段落間は空行で区切ること。\
            自然な話し言葉で書き、タグや記号は含めないこと。すべて日本語で書くこと。\
            内容を端折らず、必要な分だけ長さを惜しまずに書くこと。",
        )
    } else {
        format!(
            "You are a podcast narrator. \
            Write a complete narration script for \"{name}\" ({description}). \
            Single narrator monologue, paragraphs separated by blank lines. \
            Natural spoken language only, no tags or non-speech symbols. \
            Write entirely in BCP-47 \"{lang}\". \
            Do not cut corners — write as much length as the material warrants.",
        )
    };
    (system, source.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::llm::LLMClient as _;

    #[tokio::test]
    async fn test_generate_script_ja() {
        let pdf_bytes = std::fs::read("src/features/podcast/usecase/Lecture 4.pdf").unwrap();
        let doc = pdf_oxide::PdfDocument::from_bytes(pdf_bytes).unwrap();
        let md = doc
            .to_markdown_all(&pdf_oxide::converters::ConversionOptions::default())
            .unwrap();
        let source = format!("## Document\n{md}\n\n");

        let (system, user) = build_script_prompt(
            "Lecture 4",
            "授業資料の解説ポッドキャスト",
            "ja",
            &source,
        );

        let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
        let llm = crate::util::llm::CopilotImpl::new(github_token).unwrap();
        let script = llm
            .complete(
                crate::util::llm::CopilotImpl::MODEL_GPT_5,
                crate::util::llm::CompletionInput {
                    messages: vec![
                        crate::util::llm::Message {
                            role: crate::util::llm::Role::System,
                            content: system,
                        },
                        crate::util::llm::Message {
                            role: crate::util::llm::Role::User,
                            content: user,
                        },
                    ],
                },
            )
            .await
            .unwrap()
            .content;

        std::fs::write("src/features/podcast/usecase/script_ja.txt", &script).unwrap();

        use crate::util::tts::TTSClient as _;
        let tts = crate::util::tts::SupertonicTtsImpl::new().unwrap();
        let audio = tts
            .synthesize_dialogue(crate::util::tts::SynthesizeDialogueInput {
                script: script.clone(),
                voice: "ja:M2".to_string(),
            })
            .await
            .unwrap();
        let wav = crate::util::tts::wrap_pcm_as_wav(&audio.audio, audio.sample_rate);
        std::fs::write("src/features/podcast/usecase/script_ja.wav", &wav).unwrap();

        println!("written: script_ja.txt ({} chars), script_ja.wav ({} bytes)", script.len(), wav.len());
    }
}
