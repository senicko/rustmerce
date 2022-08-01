use actix_multipart::Field;
use actix_web::web;
use futures_util::{Future, TryStreamExt};
use std::{io::Write, pin::Pin};

use crate::error::{AppError, AppErrorType};

pub trait Storage {
    fn save_image(
        &self,
        field: Field,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + '_>>;
}

#[derive(Clone)]
pub struct StorageImpl {}

impl StorageImpl {
    pub fn new() -> StorageImpl {
        StorageImpl {}
    }
}

impl Storage for StorageImpl {
    fn save_image(
        &self,
        mut field: Field,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + '_>> {
        Box::pin(async move {
            // Check if field is of type image/jpeg
            let mime = field.content_type();

            match (mime.type_(), mime.subtype()) {
                (mime::IMAGE, mime::JPEG) => {
                    // TODO: Make file destination configurable
                    let filename = format!("{}.jpeg", uuid::Uuid::new_v4());
                    let file_path = format!("./tmp/{}", filename);

                    // Save file
                    let mut f = web::block(|| std::fs::File::create(file_path)).await??;
                    while let Some(chunk) = field.try_next().await? {
                        f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                    }

                    Ok(filename)
                }
                _ => Err(AppError {
                    cause: Some(format!(
                        "Invalid image mime type. Expected image/jpeg but got {}.",
                        mime.essence_str()
                    )),
                    message: Some("Image must be a jpeg image.".to_string()),
                    error_type: AppErrorType::Internal,
                }),
            }
        })
    }
}
