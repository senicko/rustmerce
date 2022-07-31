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
}

impl Repo for RepoImpl {
    fn get_all(&self) -> Pin<Box<dyn Future<Output = Result<Vec<Product>, AppError>> + '_>> {
        Box::pin(async move {
            let conn = &self.db_pool.get().await?;

            let stmt = conn.prepare_cached("SELECT * FROM products").await?;
            let rows = conn.query(&stmt, &[]).await?;

            rows.iter()
                .map(|r| Ok(Product::from_row_ref(r)?))
                .collect::<Result<Vec<Product>, AppError>>()
        })
    }
}
