use super::ProductInsertable;
use crate::{error::AppError, product::Product};
use async_trait::async_trait;
use deadpool_postgres::Pool;
use tokio_pg_mapper::FromTokioPostgresRow;

#[async_trait]
pub trait Repo {
    async fn get_all(&self) -> Result<Vec<Product>, AppError>;

    async fn get_by_id(&self, id: i32) -> Result<Option<Product>, AppError>;

    async fn insert(&self, data: ProductInsertable) -> Result<Product, AppError>;

    async fn delete_by_id(&self, id: i32) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct RepoImpl {
    db_pool: Pool,
}

impl RepoImpl {
    pub fn new(db_pool: Pool) -> RepoImpl {
        RepoImpl { db_pool }
    }
}

#[async_trait]
impl Repo for RepoImpl {
    async fn get_all(&self) -> Result<Vec<Product>, AppError> {
        let conn = &self.db_pool.get().await?;

        let stmt = conn.prepare_cached("SELECT * FROM products").await?;
        let rows = conn.query(&stmt, &[]).await?;

        rows.iter()
            .map(|r| Ok(Product::from_row_ref(r)?))
            .collect::<Result<Vec<Product>, AppError>>()
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<Product>, AppError> {
        let conn = &self.db_pool.get().await?;

        let stmt = conn
            .prepare_cached("SELECT * FROM products WHERE id = $1")
            .await?;

        let row = conn.query_opt(&stmt, &[&id]).await?;

        match row {
            Some(r) => Ok(Some(Product::from_row(r)?)),
            None => Ok(None),
        }
    }

    async fn insert(&self, product: ProductInsertable) -> Result<Product, AppError> {
        let conn = &self.db_pool.get().await?;

        let stmt = conn
            .prepare_cached("INSERT INTO products (name, price) VALUES ($1, $2) RETURNING *")
            .await?;

        let row = conn
            .query_one(&stmt, &[&product.name, &product.price])
            .await?;

        Ok(Product::from_row(row)?)
    }

    async fn delete_by_id(&self, id: i32) -> Result<(), AppError> {
        let conn = &self.db_pool.get().await?;

        let stmt = conn
            .prepare_cached("DELETE FROM products WHERE id = $1")
            .await?;

        conn.execute(&stmt, &[&id]).await?;

        Ok(())
    }
}
