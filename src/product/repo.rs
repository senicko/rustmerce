use super::{Asset, Product, ProductInsertable};
use crate::error::{AppError, AppErrorType};
use async_trait::async_trait;
use deadpool_postgres::{Pool, Transaction};
use futures::{stream::FuturesUnordered, TryStreamExt};
use tokio_pg_mapper::FromTokioPostgresRow;

#[async_trait]
pub trait Repo {
    async fn get_all(&self) -> Result<Vec<Product>, AppError>;
    async fn get_by_id(&self, id: i32) -> Result<Product, AppError>;
    async fn insert(&self, data: ProductInsertable) -> Result<Product, AppError>;
    async fn delete_by_id(&self, id: i32) -> Result<(), AppError>;
    async fn add_asset(&self, product_id: i32, asset_filename: &String) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct RepoImpl {
    db_pool: Pool,
}

impl RepoImpl {
    pub fn new(db_pool: Pool) -> RepoImpl {
        RepoImpl { db_pool }
    }

    async fn get_product_assets<'a>(
        &self,
        product_id: i32,
        transaction: &Transaction<'a>,
    ) -> Result<Vec<Asset>, AppError> {
        let assets_rows = transaction
            .query("SELECT * FROM assets WHERE product_id = $1", &[&product_id])
            .await?;

        Ok(assets_rows
            .iter()
            .map(|row| Ok(Asset::from_row_ref(row)?))
            .collect::<Result<Vec<Asset>, AppError>>()?)
    }
}

#[async_trait]
impl Repo for RepoImpl {
    async fn get_all(&self) -> Result<Vec<Product>, AppError> {
        let mut conn = self.db_pool.get().await?;
        let transaction = conn.transaction().await?;

        let result = async {
            let product_rows = transaction.query("SELECT * FROM products", &[]).await?;
            let transaction_ref = &transaction;

            Ok::<_, AppError>(
                product_rows
                    .iter()
                    .map(|row| async move {
                        let mut product = Product::try_from(row)?;
                        product.assets =
                            self.get_product_assets(product.id, transaction_ref).await?;

                        Ok::<_, AppError>(product)
                    })
                    .collect::<FuturesUnordered<_>>()
                    .try_collect::<Vec<_>>()
                    .await?,
            )
        }
        .await;

        match result {
            Ok(_) => transaction.commit().await?,
            Err(_) => transaction.rollback().await?,
        };

        result
    }

    async fn get_by_id(&self, id: i32) -> Result<Product, AppError> {
        let mut conn = self.db_pool.get().await?;
        let transaction = conn.transaction().await?;

        let result = async {
            let row = transaction
                .query_opt("SELECT * FROM products WHERE id = $1", &[&id])
                .await?;

            match row {
                Some(row) => {
                    let mut product = Product::try_from(row)?;
                    product.assets = self.get_product_assets(product.id, &transaction).await?;

                    Ok(product)
                }
                None => Err(AppError {
                    cause: None,
                    message: Some("Product not found".to_string()),
                    error_type: AppErrorType::NotFound,
                }),
            }
        }
        .await;

        match result {
            Ok(_) => transaction.commit().await?,
            Err(_) => transaction.rollback().await?,
        };

        result
    }

    async fn insert(&self, product: ProductInsertable) -> Result<Product, AppError> {
        let conn = self.db_pool.get().await?;

        let row = conn
            .query_one(
                "INSERT INTO products (name, price) VALUES ($1, $2) RETURNING *",
                &[&product.name, &product.price],
            )
            .await?;

        Ok(Product::try_from(row)?)
    }

    async fn delete_by_id(&self, id: i32) -> Result<(), AppError> {
        let conn = self.db_pool.get().await?;

        conn.execute("DELETE FROM products WHERE id = $1", &[&id])
            .await?;

        Ok(())
    }

    async fn add_asset(&self, product_id: i32, asset_filename: &String) -> Result<(), AppError> {
        let conn = self.db_pool.get().await?;

        conn.execute(
            "INSERT INTO assets (product_id, filename) VALUES ($1, $2)",
            &[&product_id, &asset_filename],
        )
        .await?;

        Ok(())
    }
}
