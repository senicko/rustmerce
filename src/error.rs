use actix_web::body::BoxBody;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde_json::json;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AppErrorType {
    Internal,
    NotFound,
}

#[derive(Debug)]
pub struct AppError {
    pub cause: Option<String>,
    pub message: Option<String>,
    pub error_type: AppErrorType,
}

impl AppError {
    fn client_message(&self) -> String {
        if let Some(m) = &self.message {
            return m.clone();
        }

        match &self.error_type {
            AppErrorType::Internal => "Internal Server Error".to_string(),
            AppErrorType::NotFound => "Resource Not Found".to_string(),
        }
    }
}

impl<T> From<T> for AppError
where
    T: std::error::Error,
{
    fn from(e: T) -> Self {
        AppError {
            cause: Some(e.to_string()),
            message: None,
            error_type: AppErrorType::Internal,
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match &self.error_type {
            AppErrorType::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(json!({
            "message": self.client_message()
        }))
    }
}
