#[derive(Debug)]
pub enum DomainErrorKind {
    BadInput,
    NotFound,
    Forbidden,
}

#[derive(Debug)]
pub struct DomainError {
    code: &'static str,
    description: String,
    kind: DomainErrorKind,
}

impl DomainError {
    pub fn new(code: &'static str, description: String, kind: DomainErrorKind) -> Self {
        Self {
            code,
            description,
            kind,
        }
    }
}

impl From<DomainError> for (u16, crate::server::schema::ErrorBody) {
    fn from(val: DomainError) -> Self {
        let status = match val.kind {
            DomainErrorKind::BadInput => 400,
            DomainErrorKind::NotFound => 404,
            DomainErrorKind::Forbidden => 403,
        };
        (
            status,
            crate::server::schema::ErrorBody {
                code: val.code.to_string(),
                description: val.description,
            },
        )
    }
}
