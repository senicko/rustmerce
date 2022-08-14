use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
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

#[derive(Clone)]
pub struct Storage;

impl Storage {
    pub fn new() -> Self {
        Storage
    }

    pub async fn save_image(&self, mut multipart: Multipart) -> Result<String, StorageError> {
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

    pub async fn delete_image(&self, filename: &str) -> Result<(), StorageError> {
        let file_path = format!("./assets/{}", filename);
        std::fs::remove_file(file_path)?;

        Ok(())
    }
}
