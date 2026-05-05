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

impl Into<(u16, crate::server::schema::ErrorBody)> for DomainError {
    fn into(self) -> (u16, crate::server::schema::ErrorBody) {
        let status = match self.kind {
            DomainErrorKind::BadInput => 400,
            DomainErrorKind::NotFound => 404,
            DomainErrorKind::Forbidden => 403,
        };
        (
            status,
            crate::server::schema::ErrorBody {
                code: self.code.to_string(),
                description: self.description,
            },
        )
    }
}
