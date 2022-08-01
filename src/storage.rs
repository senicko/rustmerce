use actix_multipart::Field;
use async_trait::async_trait;
use futures_util::TryStreamExt;
use std::io::Write;

use crate::error::{AppError, AppErrorType};

#[async_trait(?Send)]
pub trait Storage {
    async fn save_image(&self, field: Field) -> Result<String, AppError>;
}

#[derive(Clone)]
pub struct StorageImpl {}

impl StorageImpl {
    pub fn new() -> StorageImpl {
        StorageImpl {}
    }
}

#[async_trait(?Send)]
impl Storage for StorageImpl {
    async fn save_image(&self, mut field: Field) -> Result<String, AppError> {
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
            _ => Err(AppError {
                cause: Some(format!(
                    "Invalid image mime type. Expected image/jpeg but got {}.",
                    mime.essence_str()
                )),
                message: Some("Image must be in a jpeg/jpg format.".to_string()),
                error_type: AppErrorType::Internal,
            }),
        }
    }
}
