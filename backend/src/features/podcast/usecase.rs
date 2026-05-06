// create podcast

pub struct CreatePodcastInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub used_file_ids: Vec<uuid::Uuid>,
}

pub struct CreatePodcastOutput {
    pub podcast_id: uuid::Uuid,
}

pub async fn create_podcast(
    app: &crate::app::App,
    input: CreatePodcastInput,
) -> Result<Result<CreatePodcastOutput, crate::domain::DomainError>, anyhow::Error> {
    if let Err(err) = super::domain::PodcastName::new(input.name.clone()) {
        return Ok(Err(err));
    }
    if let Err(err) = super::domain::PodcastDescription::new(input.description.clone()) {
        return Ok(Err(err));
    }
    if input.used_file_ids.is_empty() {
        return Ok(Err(crate::domain::DomainError::new(
            "invalid_used_file_ids",
            "used_file_ids must not be empty".to_string(),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }

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

    let in_progress = app
        .podcast_request_service
        .list_by_user(input.user_id)
        .await?;
    if in_progress.len() >= super::domain::MAX_CONCURRENT_GENERATIONS {
        return Ok(Err(crate::domain::DomainError::new(
            "max_concurrent_podcast_generations_exceeded",
            format!(
                "user can have at most {} concurrent podcast generations",
                super::domain::MAX_CONCURRENT_GENERATIONS
            ),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }

    let podcast_id = uuid::Uuid::new_v4();
    let request = crate::util::podcast_request::PodcastRequest {
        podcast_id,
        user_id: input.user_id,
        project_id: input.project_id,
        name: input.name,
        description: input.description,
        used_file_ids: input.used_file_ids,
        started_at: chrono::Utc::now(),
    };

    app.podcast_request_service
        .save(&request, std::time::Duration::from_secs(3600))
        .await?;

    Ok(Ok(CreatePodcastOutput { podcast_id }))
}

// generate podcast (worker entry)

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
    let podcast_intermediate = match super::domain::Podcast::new(
        request.name,
        request.description,
        request.user_id,
        request.project_id,
        request.used_file_ids,
        Vec::new(),
        audio_storage_id,
        super::domain::PodcastOption {
            id: Some(podcast_id),
            ..Default::default()
        },
    )? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    // Step 1: 資料 → custom prompt (英語)
    let custom_prompt = app
        .llm_client
        .complete(
            crate::util::llm::CopilotImpl::MODEL_GEMINI_3_1_PRO,
            podcast_intermediate.to_custom_prompt_llm_input(&documents),
        )
        .await?
        .content;

    // Step 2: custom prompt + 資料 → English script
    let english_script = app
        .llm_client
        .complete(
            crate::util::llm::CopilotImpl::MODEL_GEMINI_3_1_PRO,
            podcast_intermediate.to_script_llm_input(&documents, &custom_prompt),
        )
        .await?
        .content;

    // Step 3: English → ユーザー言語
    let translated_script = app
        .llm_client
        .complete(
            crate::util::llm::CopilotImpl::MODEL_GEMINI_3_1_PRO,
            podcast_intermediate.to_translation_llm_input(&english_script, &user_lang),
        )
        .await?
        .content;

    let podcast_script = parse_script(&translated_script);

    // 文字数で chunk → TTS 順次呼び出し → audio bytes 連結
    const TTS_CHUNK_CHARS: usize = 10_000;
    let speakers = vec![
        crate::util::tts::SpeakerVoice {
            speaker: "Ken".to_string(),
            voice: "Puck".to_string(),
        },
        crate::util::tts::SpeakerVoice {
            speaker: "Maya".to_string(),
            voice: "Kore".to_string(),
        },
    ];
    const TTS_MAX_ATTEMPTS: u32 = 3;
    let mut chars = translated_script.chars();
    let mut audio_content_type = String::new();
    let mut audio = Vec::<u8>::new();
    loop {
        let chunk = chars.by_ref().take(TTS_CHUNK_CHARS).collect::<String>();
        if chunk.is_empty() {
            break;
        }
        let mut last_err: Option<anyhow::Error> = None;
        let mut part: Option<crate::util::tts::SynthesizedAudio> = None;
        for attempt in 0..TTS_MAX_ATTEMPTS {
            match app
                .tts_client
                .synthesize_dialogue(crate::util::tts::SynthesizeDialogueInput {
                    script: chunk.clone(),
                    speakers: speakers.clone(),
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

    // Gemini returns raw L16 PCM. Concatenate raw PCM across chunks first, then wrap
    // once with a WAV header so the stored object is browser-playable.
    let sample_rate = crate::util::tts::parse_pcm_sample_rate(&audio_content_type);
    let wav = crate::util::tts::wrap_pcm_as_wav(&audio, sample_rate);

    app.storage_service
        .put_object(audio_storage_id, "audio/wav", wav)
        .await?;

    let podcast = match super::domain::Podcast::new(
        podcast_intermediate.name.0,
        podcast_intermediate.description.0,
        podcast_intermediate.user_id,
        podcast_intermediate.project_id,
        podcast_intermediate.used_file_ids,
        podcast_script,
        audio_storage_id,
        super::domain::PodcastOption {
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

fn parse_script(script: &str) -> Vec<super::domain::PodcastScriptEntry> {
    script
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let (speaker, text) = line.split_once(':')?;
            let speaker = speaker.trim().to_string();
            let text = text.trim().to_string();
            if speaker.is_empty() || text.is_empty() {
                return None;
            }
            Some(super::domain::PodcastScriptEntry { speaker, text })
        })
        .collect::<Vec<_>>()
}

// get podcast

pub struct GetPodcastInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub podcast_id: uuid::Uuid,
}

pub struct GetPodcastOutput {
    pub view: super::query_service::GetPodcastView,
    pub audio_url: String,
}

pub async fn get_podcast(
    app: &crate::app::App,
    input: GetPodcastInput,
) -> Result<Result<GetPodcastOutput, crate::domain::DomainError>, anyhow::Error> {
    let podcast = match app
        .podcast_query_service
        .get_podcast(input.podcast_id)
        .await?
    {
        Some(p) if p.project_id == input.project_id => p,
        _ => {
            return Ok(Err(crate::domain::DomainError::new(
                "podcast_not_found",
                "podcast not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if podcast.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "podcast does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    let audio_url = app
        .storage_service
        .presign_get(
            podcast.audio_storage_id,
            std::time::Duration::from_secs(3600),
        )
        .await?
        .url;

    Ok(Ok(GetPodcastOutput {
        view: podcast,
        audio_url,
    }))
}

// list podcasts

#[derive(Clone, Copy)]
pub enum PodcastStatus {
    Generating,
    Generated,
}

pub struct PodcastListItem {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: PodcastStatus,
}

pub struct ListPodcastsCursor {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub podcast_id: uuid::Uuid,
}

pub struct ListPodcastsInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub cursor: Option<ListPodcastsCursor>,
    pub limit: u32,
}

pub struct ListPodcastsOutput {
    pub items: Vec<PodcastListItem>,
    pub next_cursor: Option<ListPodcastsCursor>,
}

pub async fn list_podcasts(
    app: &crate::app::App,
    input: ListPodcastsInput,
) -> Result<Result<ListPodcastsOutput, crate::domain::DomainError>, anyhow::Error> {
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

    let limit = input.limit;
    let mut items = Vec::<PodcastListItem>::new();

    if input.cursor.is_none() {
        let mut in_progress = app
            .podcast_request_service
            .list_by_user(input.user_id)
            .await?
            .into_iter()
            .filter(|r| r.project_id == input.project_id)
            .collect::<Vec<_>>();
        in_progress.sort_by_key(|b| std::cmp::Reverse(b.started_at));
        for req in in_progress {
            items.push(PodcastListItem {
                id: req.podcast_id,
                name: req.name,
                description: req.description,
                user_id: req.user_id,
                project_id: req.project_id,
                created_at: req.started_at,
                status: PodcastStatus::Generating,
            });
        }
    }

    let cursor = input
        .cursor
        .map(|c| super::query_service::ListPodcastsByProjectCursor {
            created_at: c.created_at,
            podcast_id: c.podcast_id,
        });
    let mut ready = app
        .podcast_query_service
        .list_podcasts_by_project(input.project_id, cursor, limit + 1)
        .await?;
    let next_cursor = if ready.len() as u32 > limit {
        ready.pop().map(|item| ListPodcastsCursor {
            created_at: item.podcast_created_at,
            podcast_id: item.id,
        })
    } else {
        None
    };
    for view in ready {
        items.push(PodcastListItem {
            id: view.id,
            name: view.name,
            description: view.description,
            user_id: view.user_id,
            project_id: view.project_id,
            created_at: view.podcast_created_at,
            status: PodcastStatus::Generated,
        });
    }

    Ok(Ok(ListPodcastsOutput { items, next_cursor }))
}

// update podcast

pub struct UpdatePodcastInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub podcast_id: uuid::Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct UpdatePodcastOutput {
    pub view: super::query_service::GetPodcastView,
    pub audio_url: String,
}

pub async fn update_podcast(
    app: &crate::app::App,
    input: UpdatePodcastInput,
) -> Result<Result<UpdatePodcastOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let mut podcast = match app
        .podcast_repository
        .find_for_update(&mut tx, input.podcast_id)
        .await?
    {
        Some(ok) if ok.project_id == input.project_id => ok,
        _ => {
            return Ok(Err(crate::domain::DomainError::new(
                "podcast_not_found",
                "podcast not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if podcast.user_id != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "podcast does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    if let Some(name) = input.name {
        match podcast.set_name(name) {
            Ok(()) => {}
            Err(err) => return Ok(Err(err)),
        }
    }
    if let Some(description) = input.description {
        match podcast.set_description(description) {
            Ok(()) => {}
            Err(err) => return Ok(Err(err)),
        }
    }

    match app.podcast_repository.save(&mut tx, &podcast).await? {
        Ok(()) => {}
        Err(err) => return Ok(Err(err)),
    }
    tx.commit().await?;

    let view = app
        .podcast_query_service
        .get_podcast(input.podcast_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("podcast not found after update"))?;

    let audio_url = app
        .storage_service
        .presign_get(view.audio_storage_id, std::time::Duration::from_secs(3600))
        .await?
        .url;

    Ok(Ok(UpdatePodcastOutput { view, audio_url }))
}

// remove podcast

pub struct RemovePodcastInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub podcast_id: uuid::Uuid,
}

pub struct RemovePodcastOutput {}

pub async fn remove_podcast(
    app: &crate::app::App,
    input: RemovePodcastInput,
) -> Result<Result<RemovePodcastOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    if let Some(podcast) = app
        .podcast_repository
        .find_for_update(&mut tx, input.podcast_id)
        .await?
        && podcast.project_id == input.project_id
    {
        if podcast.user_id != input.user_id {
            return Ok(Err(crate::domain::DomainError::new(
                "forbidden",
                "podcast does not belong to the user".to_string(),
                crate::domain::DomainErrorKind::Forbidden,
            )));
        }
        match app.podcast_repository.remove(&mut tx, podcast.id).await? {
            Ok(()) => {}
            Err(err) => return Ok(Err(err)),
        }
        tx.commit().await?;
        let _ = app.podcast_request_service.remove(input.podcast_id).await;
        return Ok(Ok(RemovePodcastOutput {}));
    }
    drop(tx);

    if let Some(request) = app.podcast_request_service.get(input.podcast_id).await?
        && request.project_id == input.project_id
    {
        if request.user_id != input.user_id {
            return Ok(Err(crate::domain::DomainError::new(
                "forbidden",
                "podcast does not belong to the user".to_string(),
                crate::domain::DomainErrorKind::Forbidden,
            )));
        }
        app.podcast_request_service.remove(input.podcast_id).await?;
        return Ok(Ok(RemovePodcastOutput {}));
    }

    Ok(Err(crate::domain::DomainError::new(
        "podcast_not_found",
        "podcast not found".to_string(),
        crate::domain::DomainErrorKind::NotFound,
    )))
}
