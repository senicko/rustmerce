use super::Category;
use deadpool_postgres::Pool;
use tokio_pg_mapper::FromTokioPostgresRow;

#[derive(thiserror::Error, Debug)]
pub enum CategoryStoreError {
    #[error("Database query failed")]
    QueryFailed(#[from] tokio_postgres::Error),

    #[error("Result mapping failed")]
    MappingFailed(#[from] tokio_pg_mapper::Error),

    #[error("Database connection failed")]
    ConnectionFailed(#[from] deadpool_postgres::PoolError),
}

#[derive(Clone)]
pub struct CategoryStore {
    db_pool: Pool,
}

impl CategoryStore {
    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }

    pub async fn get_all(&self) -> Result<Vec<Category>, CategoryStoreError> {
        let conn = self.db_pool.get().await?;

        let rows = conn
            .query("SELECT * FROM categories WHERE parent_id IS NULL", &[])
            .await?;

        rows.iter()
            .map(|row| Ok(Category::from_row_ref(row)?))
            .collect()
    }
}
