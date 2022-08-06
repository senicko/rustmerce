use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use serde_json::json;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    Internal,
    BadRequest,
}

#[derive(Debug)]
pub struct Error {
    message: String,
    kind: ErrorKind,
}

impl Error {
    pub fn new(message: String, kind: ErrorKind) -> Self {
        Error { message, kind }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server error: {}", self.message)
    }
}

impl<T> From<T> for Error
where
    T: error::Error,
{
    fn from(e: T) -> Self {
        Error {
            message: e.to_string(),
            kind: ErrorKind::Internal,
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self.kind {
            ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self.kind {
            ErrorKind::Internal => HttpResponse::build(self.status_code()).finish(),
            ErrorKind::BadRequest => HttpResponse::build(self.status_code()).json(json!({
                "message": self.message
            })),
        }
    }
}
