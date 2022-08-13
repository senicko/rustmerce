use actix_multipart::Multipart;
use actix_web::{HttpResponse, ResponseError};
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use serde_json::json;
use std::{io::Write, ops::Deref};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO operation failed")]
    Io(#[from] std::io::Error),

    #[error("Multipart error")]
    Multipart(#[from] actix_multipart::MultipartError),

    #[error("Multipart field missing")]
    MultipartFieldMissing(String),

    #[error("Invalid mime type")]
    InvalidMimeType,
}

impl ResponseError for StorageError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            StorageError::Io(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            StorageError::Multipart(_)
            | StorageError::MultipartFieldMissing(_)
            | StorageError::InvalidMimeType => actix_web::http::StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            StorageError::Io(_) => HttpResponse::build(self.status_code()).json(json!({
                "error": "Internal server error",
            })),
            StorageError::Multipart(_)
            | StorageError::MultipartFieldMissing(_)
            | StorageError::InvalidMimeType => {
                HttpResponse::build(self.status_code()).json(json!({
                    "error": "Bad request",
                }))
            }
        }
    }
}

#[async_trait(?Send)]
pub trait Storage {
    async fn save_image(&self, multipart: Multipart) -> Result<String, StorageError>;
    async fn delete_image(&self, filename: &str) -> Result<(), StorageError>;
}

#[derive(Clone)]
pub struct StorageImpl;

impl StorageImpl {
    pub fn new() -> Self {
        StorageImpl
    }
}

#[async_trait(?Send)]
impl Storage for StorageImpl {
    async fn save_image(&self, mut multipart: Multipart) -> Result<String, StorageError> {
        let field_name = "payload".to_string();

        while let Some(Ok(mut field)) = multipart.next().await {
            let content_disposition = field.content_disposition();

            if Some(field_name.deref()) != content_disposition.get_name() {
                continue;
            }

            let mime = field.content_type();

            match (mime.type_(), mime.subtype()) {
                (mime::IMAGE, mime::JPEG) | (mime::IMAGE, mime::PNG) => {
                    let filename = format!("{}.jpeg", uuid::Uuid::new_v4());
                    let file_path = format!("./assets/{}", filename);

                    let mut f = std::fs::File::create(file_path)?;
                    while let Some(chunk) = field.try_next().await? {
                        f = f.write_all(&chunk).map(|_| f)?;
                    }

                    return Ok(filename);
                }
                _ => return Err(StorageError::InvalidMimeType),
            }
        }

        Err(StorageError::MultipartFieldMissing(field_name))
    }

    async fn delete_image(&self, filename: &str) -> Result<(), StorageError> {
        let file_path = format!("./assets/{}", filename);
        std::fs::remove_file(file_path)?;

        Ok(())
    }
}
