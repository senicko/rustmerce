use super::{
    repo::{ProductRepo, ProductRepoError},
    Product, ProductInsertable,
};

#[derive(thiserror::Error, Debug)]
pub enum ProductServiceError {
    #[error("Product repository failed")]
    RepoError(#[from] ProductRepoError),
}

#[derive(Clone)]
pub struct ProductService {
    product_repo: ProductRepo,
}

// This ProductService thing feels like a little overkill, but it is there for now to follow hexagonal architecture principles.
impl ProductService {
    pub fn new(product_repo: ProductRepo) -> Self {
        ProductService { product_repo }
    }

    pub async fn get_all(&self) -> Result<Vec<Product>, ProductServiceError> {
        Ok(self.product_repo.get_all().await?)
    }

    pub async fn get_one(&self, id: i32) -> Result<Option<Product>, ProductServiceError> {
        Ok(self.product_repo.get_by_id(id).await?)
    }

    pub async fn create(&self, data: ProductInsertable) -> Result<Product, ProductServiceError> {
        Ok(self.product_repo.insert(data).await?)
    }

    pub async fn delete(&self, id: i32) -> Result<(), ProductServiceError> {
        Ok(self.product_repo.delete_by_id(id).await?)
    }

    pub async fn add_asset(
        &self,
        product_id: i32,
        asset_filename: &String,
    ) -> Result<(), ProductServiceError> {
        self.product_repo
            .add_asset(product_id, asset_filename)
            .await?;

        Ok(())
    }
}
