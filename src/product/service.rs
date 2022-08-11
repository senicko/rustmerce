use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use async_trait::async_trait;
use serde_json::json;
use thiserror::Error;

use super::{
    repo::{Repo, RepoError, RepoImpl},
    Product, ProductInsertable,
};

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Repository failed")]
    RepoError(#[from] RepoError),
}

impl ResponseError for ServiceError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ServiceError::RepoError(e) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({
            "error": "Internal server error",
        }))
    }
}

#[async_trait]
pub trait Service {
    async fn get_all(&self) -> Result<Vec<Product>, ServiceError>;
    async fn get_one(&self, id: i32) -> Result<Option<Product>, ServiceError>;
    async fn create(&self, data: ProductInsertable) -> Result<Product, ServiceError>;
    async fn delete(&self, id: i32) -> Result<(), ServiceError>;
}

#[derive(Clone)]
pub struct ServiceImpl {
    // TODO: Change this to type that accepts any struct which implements Repo trait
    repo: RepoImpl,
}

impl ServiceImpl {
    pub fn new(repo: RepoImpl) -> Self {
        ServiceImpl { repo }
    }
}

#[async_trait]
impl Service for ServiceImpl {
    async fn get_all(&self) -> Result<Vec<Product>, ServiceError> {
        Ok(self.repo.get_all().await?)
    }

    async fn get_one(&self, id: i32) -> Result<Option<Product>, ServiceError> {
        Ok(self.repo.get_by_id(id).await?)
    }

    async fn create(&self, data: ProductInsertable) -> Result<Product, ServiceError> {
        Ok(self.repo.insert(data).await?)
    }

    async fn delete(&self, id: i32) -> Result<(), ServiceError> {
        Ok(self.repo.delete_by_id(id).await?)
    }
}
