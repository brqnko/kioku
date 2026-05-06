#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    pub code: String,
    pub description: String,
}

pub struct HandlerResult<T: axum::response::IntoResponse>(
    pub Result<Result<T, crate::domain::DomainError>, anyhow::Error>,
);

impl<T: axum::response::IntoResponse> axum::response::IntoResponse for HandlerResult<T> {
    fn into_response(self) -> axum::response::Response {
        match self.0 {
            Ok(Ok(data)) => data.into_response(),
            Ok(Err(err)) => {
                let (status, body): (u16, ErrorBody) = err.into();
                let status = axum::http::StatusCode::from_u16(status)
                    .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
                (status, axum::Json(body)).into_response()
            }
            Err(err) => {
                tracing::error!("{:?}", err);
                axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
