use actix_multipart::Field;
use async_trait::async_trait;
use futures::TryStreamExt;
use std::io::Write;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO operation failed")]
    Io(#[from] std::io::Error),

    #[error("Multipart error")]
    Multipart(#[from] actix_multipart::MultipartError),

    #[error("Invalid mime type")]
    InvalidMimeType,
}

#[async_trait(?Send)]
pub trait Storage {
    async fn save_image(&self, field: Field) -> Result<String, StorageError>;
    async fn delete_image(&self, filename: &str) -> Result<(), StorageError>;
}

#[derive(Clone)]
pub struct StorageImpl;

#[async_trait(?Send)]
impl Storage for StorageImpl {
    async fn save_image(&self, mut field: Field) -> Result<String, StorageError> {
        let mime = field.content_type();

        match (mime.type_(), mime.subtype()) {
            (mime::IMAGE, mime::JPEG) => {
                // TODO: Make file destination configurable
                let filename = format!("{}.jpeg", uuid::Uuid::new_v4());
                let file_path = format!("./assets/{}", filename);

                let mut f = std::fs::File::create(file_path)?;
                while let Some(chunk) = field.try_next().await? {
                    f = f.write_all(&chunk).map(|_| f)?;
                }

                Ok(filename)
            }
            _ => Err(StorageError::InvalidMimeType),
        }
    }

    async fn delete_image(&self, filename: &str) -> Result<(), StorageError> {
        let file_path = format!("./assets/{}", filename);
        std::fs::remove_file(file_path)?;

        Ok(())
    }
}
