pub enum DomainErrorKind {
    BadInput,
}

pub struct DomainError {
    code: &'static str,
    description: String,
    kind: DomainErrorKind,
}

impl DomainError {
    pub fn new(code: &'static str, description: String, kind: DomainErrorKind) -> Self {
        Self { code, description, kind }
    }
}
