pub struct ProjectName(pub String);

impl ProjectName {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 256 {
            return Err(crate::domain::DomainError::new(
                "invalid_project_name",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

pub struct ProjectDescription(pub String);

impl ProjectDescription {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 512 {
            return Err(crate::domain::DomainError::new(
                "invalid_project_description",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

pub struct Project {
    pub id: uuid::Uuid,
    pub created_by: uuid::Uuid,
    pub name: ProjectName,
    pub description: ProjectDescription,
    pub indexed_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_file_id: uuid::Uuid,
}

#[derive(Default)]
pub struct ProjectOption {
    pub id: Option<uuid::Uuid>,
    pub indexed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_seen_file_id: Option<uuid::Uuid>,
}

impl Project {
    pub fn new(
        created_by: uuid::Uuid,
        name: String,
        description: String,
        option: ProjectOption,
    ) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
        let name = match ProjectName::new(name) {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };
        let description = match ProjectDescription::new(description) {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };

        Ok(Ok(Self {
            id: option.id.unwrap_or(uuid::Uuid::new_v4()),
            created_by,
            name,
            description,
            indexed_at: option.indexed_at.unwrap_or(chrono::Utc::now()),
            last_seen_at: option.last_seen_at.unwrap_or(chrono::Utc::now()),
            last_seen_file_id: option.last_seen_file_id.unwrap_or(uuid::Uuid::nil()),
        }))
    }

    pub fn set_name(&mut self, name: String) -> Result<(), crate::domain::DomainError> {
        self.name = ProjectName::new(name)?;
        self.last_seen_at = chrono::Utc::now();
        Ok(())
    }

    pub fn set_description(
        &mut self,
        description: String,
    ) -> Result<(), crate::domain::DomainError> {
        self.description = ProjectDescription::new(description)?;
        self.last_seen_at = chrono::Utc::now();
        Ok(())
    }

    pub fn set_last_seen(&mut self, file_id: uuid::Uuid) {
        self.last_seen_at = chrono::Utc::now();
        self.last_seen_file_id = file_id;
    }

    pub fn update_last_seen_at(&mut self) {
        self.last_seen_at = chrono::Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("project".to_string(), true)]
    #[case("a".repeat(256), true)]
    #[case("a".repeat(257), false)]
    fn project_name_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(ProjectName::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("desc".to_string(), true)]
    #[case("a".repeat(512), true)]
    #[case("a".repeat(513), false)]
    fn project_description_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(ProjectDescription::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("my project".to_string(), "desc".to_string(), true, true)]
    #[case("a".repeat(257), "desc".to_string(), true, false)]
    #[case("my project".to_string(), "a".repeat(513), true, false)]
    fn project_new(
        #[case] name: String,
        #[case] description: String,
        #[case] outer_ok: bool,
        #[case] inner_ok: bool,
    ) {
        let result = Project::new(
            uuid::Uuid::new_v4(),
            name,
            description,
            ProjectOption::default(),
        );
        assert_eq!(result.is_ok(), outer_ok);
        if outer_ok {
            assert_eq!(result.unwrap().is_ok(), inner_ok);
        }
    }

    fn make_project() -> Project {
        Project::new(
            uuid::Uuid::new_v4(),
            "my project".to_string(),
            "desc".to_string(),
            ProjectOption::default(),
        )
        .unwrap()
        .unwrap()
    }

    #[rstest]
    #[case("new name".to_string(), true)]
    #[case("a".repeat(256), true)]
    #[case("a".repeat(257), false)]
    fn project_set_name(#[case] input: String, #[case] should_ok: bool) {
        let mut project = make_project();
        assert_eq!(project.set_name(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("new desc".to_string(), true)]
    #[case("a".repeat(512), true)]
    #[case("a".repeat(513), false)]
    fn project_set_description(#[case] input: String, #[case] should_ok: bool) {
        let mut project = make_project();
        assert_eq!(project.set_description(input).is_ok(), should_ok);
    }

    #[test]
    fn project_set_last_seen_updates_file_id() {
        let mut project = make_project();
        let file_id = uuid::Uuid::new_v4();
        project.set_last_seen(file_id);
        assert_eq!(project.last_seen_file_id, file_id);
    }
}
