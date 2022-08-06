// use crate::product::Product;
// use actix_web::Error;
// use async_trait::async_trait;

// use super::repo::{Repo, RepoImpl};

// #[async_trait]
// pub trait Service {
//     async fn get_all(&self) -> Result<Vec<Product>, Box<dyn Error>>;
// }

// pub struct ServiceImpl {
//     repo: RepoImpl,
// }

// impl ServiceImpl {
//     fn new(repo: RepoImpl) -> Self {
//         ServiceImpl { repo }
//     }
// }

// #[async_trait]
// impl Service for ServiceImpl {
//     async fn get_all(&self) -> Result<Vec<Product>, Error> {
//         Ok(self.repo.get_all().await?)
//     }
// }
