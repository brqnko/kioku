// create project

pub struct CreateProjectInput {
    pub user_id: uuid::Uuid,
    pub name: String,
    pub description: String,
}

pub struct CreateProjectOutput {
    pub project: super::domain::Project,
}

pub async fn create_project(
    app: &crate::app::App,
    input: CreateProjectInput,
) -> Result<Result<CreateProjectOutput, crate::domain::DomainError>, anyhow::Error> {
    let project = match super::domain::Project::new(
        input.user_id,
        input.name,
        input.description,
        super::domain::ProjectOption {
            ..Default::default()
        },
    )? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    let mut tx = app.pool.begin().await?;

    match app.project_repository.save(&mut tx, &project).await? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    };

    tx.commit().await?;

    Ok(Ok(CreateProjectOutput { project }))
}

// list projects

pub struct ListProjectsInput {
    pub user_id: uuid::Uuid,
    pub order: super::query_service::ListProjectsByUserIdOrder,
    pub cursor: Option<super::query_service::ListProjectsByUserIdCursor>,
    pub limit: u32,
}

pub struct ListProjectsOutput {
    pub items: Vec<super::query_service::ListProjectsByUserIdView>,
    pub next_cursor: Option<super::query_service::ListProjectsByUserIdCursor>,
}

pub async fn list_projects(
    app: &crate::app::App,
    input: ListProjectsInput,
) -> Result<Result<ListProjectsOutput, crate::domain::DomainError>, anyhow::Error> {
    if input.limit > 32 {
        return Ok(Err(crate::domain::DomainError::new(
            "invalid_limit",
            "limit must be 32 or less".to_string(),
            crate::domain::DomainErrorKind::BadInput,
        )));
    }

    let mut rows = app
        .project_query_service
        .list_projects_by_user_id(input.user_id, input.order, input.cursor, input.limit + 1)
        .await?;

    let next_cursor = if rows.len() as u32 > input.limit {
        rows.pop()
            .map(|r| super::query_service::ListProjectsByUserIdCursor {
                last_seen_at: r.last_seen_at,
                project_id: r.id,
            })
    } else {
        None
    };

    Ok(Ok(ListProjectsOutput {
        items: rows,
        next_cursor,
    }))
}

// get project

pub struct GetProjectInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
}

pub struct GetProjectOutput {
    pub project: super::domain::Project,
}

pub async fn get_project(
    app: &crate::app::App,
    input: GetProjectInput,
) -> Result<Result<GetProjectOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let mut project = match app
        .project_repository
        .find_for_update(&mut tx, input.project_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "project_not_found",
                "project not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if project.created_by != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "project does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    project.update_last_seen_at();

    match app.project_repository.save(&mut tx, &project).await? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(GetProjectOutput { project }))
}

// update project

pub struct UpdateProjectInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct UpdateProjectOutput {
    pub project: super::domain::Project,
}

pub async fn update_project(
    app: &crate::app::App,
    input: UpdateProjectInput,
) -> Result<Result<UpdateProjectOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let mut project = match app
        .project_repository
        .find_for_update(&mut tx, input.project_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "project_not_found",
                "project not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if project.created_by != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "project does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    if let Some(name) = input.name {
        match project.set_name(name) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        }
    }

    if let Some(description) = input.description {
        match project.set_description(description) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        }
    }

    match app.project_repository.save(&mut tx, &project).await? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(UpdateProjectOutput { project }))
}

// remove project

pub struct RemoveProjectInput {
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
}

pub struct RemoveProjectOutput {}

pub async fn remove_project(
    app: &crate::app::App,
    input: RemoveProjectInput,
) -> Result<Result<RemoveProjectOutput, crate::domain::DomainError>, anyhow::Error> {
    let mut tx = app.pool.begin().await?;

    let project = match app
        .project_repository
        .find_for_update(&mut tx, input.project_id)
        .await?
    {
        Some(ok) => ok,
        None => {
            return Ok(Err(crate::domain::DomainError::new(
                "project_not_found",
                "project not found".to_string(),
                crate::domain::DomainErrorKind::NotFound,
            )));
        }
    };

    if project.created_by != input.user_id {
        return Ok(Err(crate::domain::DomainError::new(
            "forbidden",
            "project does not belong to the user".to_string(),
            crate::domain::DomainErrorKind::Forbidden,
        )));
    }

    match app.project_repository.remove(&mut tx, project.id).await? {
        Ok(ok) => ok,
        Err(err) => return Ok(Err(err)),
    }

    tx.commit().await?;

    Ok(Ok(RemoveProjectOutput {}))
}
