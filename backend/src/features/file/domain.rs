pub struct FileName(pub String);

impl FileName {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 256 {
            return Err(crate::domain::DomainError::new(
                "invalid_file_name",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

pub struct FileDescription(pub String);

impl FileDescription {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 1024 {
            return Err(crate::domain::DomainError::new(
                "invalid_file_description",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

pub enum StorageType {
    Object = 0,
    Text = 1,
}

impl TryFrom<u8> for StorageType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Object),
            1 => Ok(Self::Text),
            other => Err(anyhow::anyhow!("unknown storage_type: {other}")),
        }
    }
}

pub enum ParentId {
    Project(uuid::Uuid),
    Folder(uuid::Uuid),
}

impl ParentId {
    pub fn id(&self) -> uuid::Uuid {
        match self {
            Self::Project(id) | Self::Folder(id) => *id,
        }
    }

    pub fn kind(&self) -> u8 {
        match self {
            Self::Project(_) => 0,
            Self::Folder(_) => 1,
        }
    }

    pub fn from_raw(id: uuid::Uuid, kind: u8) -> Result<Self, crate::domain::DomainError> {
        match kind {
            0 => Ok(Self::Project(id)),
            1 => Ok(Self::Folder(id)),
            other => Err(crate::domain::DomainError::new(
                "invalid_parent_kind",
                format!("unknown parent kind: {other}"),
                crate::domain::DomainErrorKind::BadInput,
            )),
        }
    }
}

pub enum ContentType {
    ApplicationPdf,
    ImagePng,
}

impl ContentType {
    pub fn as_mime(&self) -> &'static str {
        match self {
            Self::ApplicationPdf => "application/pdf",
            Self::ImagePng => "image/png",
        }
    }

    pub fn from_mime(s: &str) -> Result<Self, crate::domain::DomainError> {
        match s {
            "application/pdf" => Ok(Self::ApplicationPdf),
            "image/png" => Ok(Self::ImagePng),
            other => Err(crate::domain::DomainError::new(
                "invalid_content_type",
                format!("unsupported content type: {other}"),
                crate::domain::DomainErrorKind::BadInput,
            )),
        }
    }
}

pub struct OriginalText(pub String);

impl OriginalText {
    pub const MAX_BYTES: usize = 512;

    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > Self::MAX_BYTES {
            return Err(crate::domain::DomainError::new(
                "invalid_original_text",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }
        Ok(Self(string))
    }
}

pub struct Embedding(pub Vec<f32>);

impl Embedding {
    pub const DIM: usize = 1024;

    pub fn new(values: Vec<f32>) -> Result<Self, crate::domain::DomainError> {
        if values.len() != Self::DIM {
            return Err(crate::domain::DomainError::new(
                "invalid_embedding_dim",
                format!("embedding dim must be {}: got {}", Self::DIM, values.len()),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }
        Ok(Self(values))
    }
}

pub struct FileEmbedding {
    pub id: uuid::Uuid,
    pub file_id: uuid::Uuid,
    pub original_text: OriginalText,
    pub embedding: Embedding,
    pub indexed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Default)]
pub struct FileEmbeddingOption {
    pub id: Option<uuid::Uuid>,
    pub indexed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl FileEmbedding {
    pub fn new(
        file_id: uuid::Uuid,
        original_text: String,
        embedding: Vec<f32>,
        option: FileEmbeddingOption,
    ) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
        let original_text = match OriginalText::new(original_text) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        };
        let embedding = match Embedding::new(embedding) {
            Ok(ok) => ok,
            Err(err) => return Ok(Err(err)),
        };

        Ok(Ok(Self {
            id: option.id.unwrap_or(uuid::Uuid::new_v4()),
            file_id,
            original_text,
            embedding,
            indexed_at: option.indexed_at.unwrap_or(chrono::Utc::now()),
        }))
    }
}

pub struct Text(pub String);

impl Text {
    // 32 MiB
    pub const MAX_BYTES: usize = 32 * 1024 * 1024;

    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > Self::MAX_BYTES {
            return Err(crate::domain::DomainError::new(
                "text_too_large",
                format!("text size exceeds maximum: {len} > {}", Self::MAX_BYTES),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

pub struct ContentLength(pub i64);

impl ContentLength {
    // 32 MiB
    pub const MAX: i64 = 32 * 1024 * 1024;

    pub fn new(value: i64) -> Result<Self, crate::domain::DomainError> {
        if value <= 0 {
            return Err(crate::domain::DomainError::new(
                "invalid_content_length",
                format!("content length must be positive: {value}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }
        if value > Self::MAX {
            return Err(crate::domain::DomainError::new(
                "content_length_too_large",
                format!("content length exceeds maximum: {value} > {}", Self::MAX),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(value))
    }
}

pub struct File {
    pub id: uuid::Uuid,
    pub name: FileName,
    pub description: FileDescription,
    pub user_id: uuid::Uuid,
    pub storage_type: StorageType,
    pub storage_id: uuid::Uuid,
    pub file_size: u64,
    pub parent: ParentId,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Default)]
pub struct FileOption {
    pub id: Option<uuid::Uuid>,
    pub uploaded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub changed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl File {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_id: uuid::Uuid,
        name: String,
        description: String,
        storage_type: StorageType,
        storage_id: uuid::Uuid,
        file_size: u64,
        parent: ParentId,
        option: FileOption,
    ) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
        let name = match FileName::new(name) {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };
        let description = match FileDescription::new(description) {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };

        Ok(Ok(Self {
            id: option.id.unwrap_or(uuid::Uuid::new_v4()),
            name,
            description,
            user_id,
            storage_type,
            storage_id,
            file_size,
            parent,
            uploaded_at: option.uploaded_at.unwrap_or(chrono::Utc::now()),
            changed_at: option.changed_at.unwrap_or(chrono::Utc::now()),
        }))
    }

    pub fn set_name(&mut self, name: String) -> Result<(), crate::domain::DomainError> {
        self.name = FileName::new(name)?;
        self.changed_at = chrono::Utc::now();
        Ok(())
    }

    pub fn set_description(
        &mut self,
        description: String,
    ) -> Result<(), crate::domain::DomainError> {
        self.description = FileDescription::new(description)?;
        self.changed_at = chrono::Utc::now();
        Ok(())
    }

    pub fn set_text_content(&mut self, text: &Text) -> Result<(), crate::domain::DomainError> {
        if !matches!(self.storage_type, StorageType::Text) {
            return Err(crate::domain::DomainError::new(
                "not_text_file",
                "file is not text-storage type".to_string(),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }
        self.file_size = text.0.len() as u64;
        self.changed_at = chrono::Utc::now();
        Ok(())
    }
}

pub struct FolderName(pub String);

impl FolderName {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 256 {
            return Err(crate::domain::DomainError::new(
                "invalid_folder_name",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

pub struct FolderDescription(pub String);

impl FolderDescription {
    pub fn new(string: String) -> Result<Self, crate::domain::DomainError> {
        let len = string.len();
        if len > 1024 {
            return Err(crate::domain::DomainError::new(
                "invalid_folder_description",
                format!("input len is too long: {len}"),
                crate::domain::DomainErrorKind::BadInput,
            ));
        }

        Ok(Self(string))
    }
}

pub struct Folder {
    pub id: uuid::Uuid,
    pub parent: ParentId,
    pub depth: u8,
    pub name: FolderName,
    pub description: FolderDescription,
    pub user_id: uuid::Uuid,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Default)]
pub struct FolderOption {
    pub id: Option<uuid::Uuid>,
    pub uploaded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub changed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Folder {
    pub const MAX_DEPTH: u8 = 4;

    pub fn new(
        user_id: uuid::Uuid,
        parent: ParentId,
        depth: u8,
        name: String,
        description: String,
        option: FolderOption,
    ) -> Result<Result<Self, crate::domain::DomainError>, anyhow::Error> {
        if depth > Self::MAX_DEPTH {
            return Ok(Err(crate::domain::DomainError::new(
                "folder_depth_too_deep",
                format!(
                    "folder depth exceeds maximum: {depth} > {}",
                    Self::MAX_DEPTH
                ),
                crate::domain::DomainErrorKind::BadInput,
            )));
        }

        let name = match FolderName::new(name) {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };
        let description = match FolderDescription::new(description) {
            Ok(ok) => ok,
            Err(err) => {
                return Ok(Err(err));
            }
        };

        Ok(Ok(Self {
            id: option.id.unwrap_or(uuid::Uuid::new_v4()),
            parent,
            depth,
            name,
            description,
            user_id,
            uploaded_at: option.uploaded_at.unwrap_or(chrono::Utc::now()),
            changed_at: option.changed_at.unwrap_or(chrono::Utc::now()),
        }))
    }

    pub fn set_name(&mut self, name: String) -> Result<(), crate::domain::DomainError> {
        self.name = FolderName::new(name)?;
        self.changed_at = chrono::Utc::now();
        Ok(())
    }

    pub fn set_description(
        &mut self,
        description: String,
    ) -> Result<(), crate::domain::DomainError> {
        self.description = FolderDescription::new(description)?;
        self.changed_at = chrono::Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("file.txt".to_string(), true)]
    #[case("a".repeat(256), true)]
    #[case("a".repeat(257), false)]
    fn file_name_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(FileName::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("application/pdf", true)]
    #[case("image/png", true)]
    #[case("image/jpeg", false)]
    #[case("audio/mpeg", false)]
    #[case("text/markdown", false)]
    #[case("", false)]
    fn content_type_from_mime(#[case] input: &str, #[case] should_ok: bool) {
        assert_eq!(ContentType::from_mime(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case(1, true)]
    #[case(ContentLength::MAX, true)]
    #[case(ContentLength::MAX + 1, false)]
    #[case(0, false)]
    #[case(-1, false)]
    fn content_length_new(#[case] input: i64, #[case] should_ok: bool) {
        assert_eq!(ContentLength::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case(0, true)]
    #[case(Text::MAX_BYTES, true)]
    #[case(Text::MAX_BYTES + 1, false)]
    fn text_new(#[case] input: usize, #[case] should_ok: bool) {
        assert_eq!(Text::new("a".repeat(input)).is_ok(), should_ok);
    }

    #[rstest]
    #[case("desc".to_string(), true)]
    #[case("a".repeat(1024), true)]
    #[case("a".repeat(1025), false)]
    fn file_description_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(FileDescription::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("file.txt".to_string(), "desc".to_string(), true, true)]
    #[case("a".repeat(257), "desc".to_string(), true, false)]
    #[case("file.txt".to_string(), "a".repeat(1025), true, false)]
    fn file_new(
        #[case] name: String,
        #[case] description: String,
        #[case] outer_ok: bool,
        #[case] inner_ok: bool,
    ) {
        let result = File::new(
            uuid::Uuid::new_v4(),
            name,
            description,
            StorageType::Object,
            uuid::Uuid::new_v4(),
            0,
            ParentId::Project(uuid::Uuid::new_v4()),
            FileOption::default(),
        );
        assert_eq!(result.is_ok(), outer_ok);
        if outer_ok {
            assert_eq!(result.unwrap().is_ok(), inner_ok);
        }
    }

    fn make_file() -> File {
        File::new(
            uuid::Uuid::new_v4(),
            "file.txt".to_string(),
            "desc".to_string(),
            StorageType::Object,
            uuid::Uuid::new_v4(),
            0,
            ParentId::Project(uuid::Uuid::new_v4()),
            FileOption::default(),
        )
        .unwrap()
        .unwrap()
    }

    #[rstest]
    #[case("new.txt".to_string(), true)]
    #[case("a".repeat(256), true)]
    #[case("a".repeat(257), false)]
    fn file_set_name(#[case] input: String, #[case] should_ok: bool) {
        let mut file = make_file();
        assert_eq!(file.set_name(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("new desc".to_string(), true)]
    #[case("a".repeat(1024), true)]
    #[case("a".repeat(1025), false)]
    fn file_set_description(#[case] input: String, #[case] should_ok: bool) {
        let mut file = make_file();
        assert_eq!(file.set_description(input).is_ok(), should_ok);
    }

    fn make_text_file() -> File {
        File::new(
            uuid::Uuid::new_v4(),
            "note.md".to_string(),
            "desc".to_string(),
            StorageType::Text,
            uuid::Uuid::new_v4(),
            0,
            ParentId::Project(uuid::Uuid::new_v4()),
            FileOption::default(),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn file_set_text_content_updates_size() {
        let mut file = make_text_file();
        let text = Text::new("hello".to_string()).unwrap();
        assert!(file.set_text_content(&text).is_ok());
        assert_eq!(file.file_size, 5);
    }

    #[test]
    fn file_set_text_content_rejects_object_file() {
        let mut file = make_file();
        let text = Text::new("hello".to_string()).unwrap();
        assert!(file.set_text_content(&text).is_err());
    }

    #[rstest]
    #[case(0, true)]
    #[case(1, true)]
    #[case(2, false)]
    fn parent_id_from_raw(#[case] kind: u8, #[case] should_ok: bool) {
        assert_eq!(
            ParentId::from_raw(uuid::Uuid::new_v4(), kind).is_ok(),
            should_ok
        );
    }

    #[rstest]
    #[case("folder".to_string(), true)]
    #[case("a".repeat(256), true)]
    #[case("a".repeat(257), false)]
    fn folder_name_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(FolderName::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("desc".to_string(), true)]
    #[case("a".repeat(1024), true)]
    #[case("a".repeat(1025), false)]
    fn folder_description_new(#[case] input: String, #[case] should_ok: bool) {
        assert_eq!(FolderDescription::new(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("folder".to_string(), "desc".to_string(), 0, true, true)]
    #[case("a".repeat(257), "desc".to_string(), 0, true, false)]
    #[case("folder".to_string(), "a".repeat(1025), 0, true, false)]
    #[case("folder".to_string(), "desc".to_string(), Folder::MAX_DEPTH, true, true)]
    #[case("folder".to_string(), "desc".to_string(), Folder::MAX_DEPTH + 1, true, false)]
    fn folder_new(
        #[case] name: String,
        #[case] description: String,
        #[case] depth: u8,
        #[case] outer_ok: bool,
        #[case] inner_ok: bool,
    ) {
        let result = Folder::new(
            uuid::Uuid::new_v4(),
            ParentId::Project(uuid::Uuid::new_v4()),
            depth,
            name,
            description,
            FolderOption::default(),
        );
        assert_eq!(result.is_ok(), outer_ok);
        if outer_ok {
            assert_eq!(result.unwrap().is_ok(), inner_ok);
        }
    }

    fn make_folder() -> Folder {
        Folder::new(
            uuid::Uuid::new_v4(),
            ParentId::Project(uuid::Uuid::new_v4()),
            0,
            "folder".to_string(),
            "desc".to_string(),
            FolderOption::default(),
        )
        .unwrap()
        .unwrap()
    }

    #[rstest]
    #[case("new folder".to_string(), true)]
    #[case("a".repeat(256), true)]
    #[case("a".repeat(257), false)]
    fn folder_set_name(#[case] input: String, #[case] should_ok: bool) {
        let mut folder = make_folder();
        assert_eq!(folder.set_name(input).is_ok(), should_ok);
    }

    #[rstest]
    #[case("new desc".to_string(), true)]
    #[case("a".repeat(1024), true)]
    #[case("a".repeat(1025), false)]
    fn folder_set_description(#[case] input: String, #[case] should_ok: bool) {
        let mut folder = make_folder();
        assert_eq!(folder.set_description(input).is_ok(), should_ok);
    }
}
