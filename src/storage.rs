use crate::error::{Error, ErrorKind};
use actix_multipart::Field;
use async_trait::async_trait;
use futures::TryStreamExt;
use std::io::Write;

#[async_trait(?Send)]
pub trait Storage {
    async fn save_image(&self, field: Field) -> Result<String, Error>;
    async fn delete_image(&self, filename: &str) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct StorageImpl;

#[async_trait(?Send)]
impl Storage for StorageImpl {
    async fn save_image(&self, mut field: Field) -> Result<String, Error> {
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
            _ => Err(Error::new(
                "Invalid mime type".to_string(),
                ErrorKind::BadRequest,
            )),
        }
    }

    async fn delete_image(&self, filename: &str) -> Result<(), Error> {
        let file_path = format!("./assets/{}", filename);
        std::fs::remove_file(file_path)?;

        Ok(())
    }
}
