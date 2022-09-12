use super::{Asset, Product, ProductInsertable};
use deadpool_postgres::{Pool, Transaction};
use futures::{stream::FuturesUnordered, TryStreamExt};
use tokio_pg_mapper::FromTokioPostgresRow;

#[derive(thiserror::Error, Debug)]
pub enum ProductStoreError {
    #[error("Database query failed")]
    QueryFailed(#[from] tokio_postgres::Error),

    #[error("Result mapping failed")]
    MappingFailed(#[from] tokio_pg_mapper::Error),

    #[error("Database connection failed")]
    ConnectionFailed(#[from] deadpool_postgres::PoolError),
}

#[derive(Clone)]
pub struct ProductStore {
    db_pool: Pool,
}

impl ProductStore {
    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }

    async fn get_product_assets<'a>(
        &self,
        product_id: i32,
        transaction: &Transaction<'a>,
    ) -> Result<Vec<Asset>, ProductStoreError> {
        let assets_rows = transaction
            .query("SELECT * FROM assets WHERE product_id = $1", &[&product_id])
            .await?;

        assets_rows
            .iter()
            .map(|row| Asset::from_row_ref(row).map_err(|e| ProductStoreError::MappingFailed(e)))
            .collect()
    }

    pub async fn get_all(&self) -> Result<Vec<Product>, ProductStoreError> {
        let mut conn = self.db_pool.get().await?;
        let transaction = conn.transaction().await?;

        let result = async {
            let product_rows = transaction.query("SELECT * FROM products", &[]).await?;
            let transaction_ref = &transaction;

            Ok(product_rows
                .iter()
                .map(|row| async move {
                    let mut product = Product::try_from(row)?;
                    product.assets = self.get_product_assets(product.id, transaction_ref).await?;
                    Ok::<_, ProductStoreError>(product)
                })
                .collect::<FuturesUnordered<_>>()
                .try_collect()
                .await?)
        }
        .await;

        if result.is_err() {
            transaction.rollback().await?;
        } else {
            transaction.commit().await?;
        }

        result
    }

    pub async fn get_one(&self, id: i32) -> Result<Option<Product>, ProductStoreError> {
        let mut conn = self.db_pool.get().await?;
        let transaction = conn.transaction().await?;

        let result = async {
            let row = transaction
                .query_opt("SELECT * FROM products WHERE id = $1", &[&id])
                .await?;

            match row {
                Some(row) => {
                    let mut product = Product::try_from(&row)?;
                    product.assets = self.get_product_assets(product.id, &transaction).await?;

                    Ok(Some(product))
                }
                None => Ok(None),
            }
        }
        .await;

        if result.is_err() {
            transaction.rollback().await?;
        } else {
            transaction.commit().await?;
        }

        result
    }

    pub async fn insert(&self, product: ProductInsertable) -> Result<Product, ProductStoreError> {
        let conn = self.db_pool.get().await?;

        let row = conn
            .query_one(
                "INSERT INTO products (name, price) VALUES ($1, $2) RETURNING *",
                &[&product.name, &product.price],
            )
            .await?;

        Ok(Product::try_from(&row)?)
    }

    pub async fn update(
        &self,
        id: i32,
        product: ProductInsertable,
    ) -> Result<(), ProductStoreError> {
        let conn = self.db_pool.get().await?;

        conn.execute(
            "UPDATE products SET name = $1, price = $2 WHERE id = $3 RETURNING *",
            &[&product.name, &product.price, &id],
        )
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: i32) -> Result<(), ProductStoreError> {
        let conn = self.db_pool.get().await?;

        conn.execute("DELETE FROM products WHERE id = $1", &[&id])
            .await?;

        Ok(())
    }

    pub async fn add_asset(
        &self,
        product_id: i32,
        asset_filename: &String,
    ) -> Result<Asset, ProductStoreError> {
        let conn = self.db_pool.get().await?;

        let row = conn
            .query_one(
                "INSERT INTO assets (product_id, filename) VALUES ($1, $2) RETURNING *",
                &[&product_id, &asset_filename],
            )
            .await?;

        Ok(Asset::from_row(row)?)
    }
}
