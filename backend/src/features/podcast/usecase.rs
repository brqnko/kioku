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

    let lock_name = format!("podcast_quota:project:{}", input.project_id.as_simple());
    let mut tx = app.pool.begin().await?;
    app.locker.acquire(&mut tx, &lock_name, 10).await?;

    let work: Result<Result<CreatePodcastOutput, crate::domain::DomainError>, anyhow::Error> =
        async move {
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

            let in_progress_for_project = in_progress
                .iter()
                .filter(|r| r.project_id == input.project_id)
                .count();
            let persisted_for_project = app
                .podcast_query_service
                .count_by_project(input.project_id)
                .await? as usize;
            if persisted_for_project + in_progress_for_project
                >= super::domain::MAX_PODCASTS_PER_PROJECT
            {
                return Ok(Err(crate::domain::DomainError::new(
                    "podcast_per_project_quota_exceeded",
                    format!(
                        "project can have at most {} podcasts",
                        super::domain::MAX_PODCASTS_PER_PROJECT
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
        .await;

    app.locker.release(&mut tx, &lock_name).await?;
    drop(tx);

    work
}

pub mod generate;
pub use generate::{GeneratePodcastInput, GeneratePodcastOutput, generate_podcast};

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
            std::time::Duration::from_secs(60 * 60),
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
        .presign_get(view.audio_storage_id, std::time::Duration::from_secs(60 * 60))
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
