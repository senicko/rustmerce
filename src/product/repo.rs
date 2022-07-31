use super::ProductInsertable;
use crate::{error::AppError, product::Product};
use deadpool_postgres::Pool;
use std::future::Future;
use std::pin::Pin;
use tokio_pg_mapper::FromTokioPostgresRow;

#[derive(Clone)]
pub struct RepoImpl {
    db_pool: Pool,
}

impl RepoImpl {
    pub fn new(db_pool: Pool) -> RepoImpl {
        RepoImpl { db_pool }
    }
}

pub trait Repo {
    fn get_all(&self) -> Pin<Box<dyn Future<Output = Result<Vec<Product>, AppError>> + '_>>;

    fn get_by_id(
        &self,
        id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Product>, AppError>> + '_>>;

    fn insert(
        &self,
        data: ProductInsertable,
    ) -> Pin<Box<dyn Future<Output = Result<Product, AppError>> + '_>>;

    fn delete_by_id(&self, id: i32) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + '_>>;
}

impl Repo for RepoImpl {
    fn get_all(&self) -> Pin<Box<dyn Future<Output = Result<Vec<Product>, AppError>> + '_>> {
        Box::pin(async {
            let conn = &self.db_pool.get().await?;

            let stmt = conn.prepare_cached("SELECT * FROM products").await?;
            let rows = conn.query(&stmt, &[]).await?;

            rows.iter()
                .map(|r| Ok(Product::from_row_ref(r)?))
                .collect::<Result<Vec<Product>, AppError>>()
        })
    }

    fn get_by_id(
        &self,
        id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Product>, AppError>> + '_>> {
        Box::pin(async move {
            let conn = &self.db_pool.get().await?;

            let stmt = conn
                .prepare_cached("SELECT * FROM products WHERE id = $1")
                .await?;

            let row = conn.query_opt(&stmt, &[&id]).await?;

            match row {
                Some(r) => Ok(Some(Product::from_row(r)?)),
                None => Ok(None),
            }
        })
    }

    fn insert(
        &self,
        product: ProductInsertable,
    ) -> Pin<Box<dyn Future<Output = Result<Product, AppError>> + '_>> {
        Box::pin(async move {
            let conn = &self.db_pool.get().await?;

            let stmt = conn
                .prepare_cached("INSERT INTO products (name, price) VALUES ($1, $2) RETURNING *")
                .await?;

            let row = conn
                .query_one(&stmt, &[&product.name, &product.price])
                .await?;

            Ok(Product::from_row(row)?)
        })
    }

    fn delete_by_id(&self, id: i32) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + '_>> {
        Box::pin(async move {
            let conn = &self.db_pool.get().await?;

            let stmt = conn
                .prepare_cached("DELETE FROM products WHERE id = $1")
                .await?;

            conn.execute(&stmt, &[&id]).await?;

            Ok(())
        })
    }
}
