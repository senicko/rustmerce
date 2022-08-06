use crate::error::Error;

use super::{Asset, Product, ProductInsertable};
use async_trait::async_trait;
use deadpool_postgres::{Pool, Transaction};
use futures::{stream::FuturesUnordered, TryStreamExt};
use tokio_pg_mapper::FromTokioPostgresRow;

#[async_trait]
pub trait Repo {
    async fn get_all(&self) -> Result<Vec<Product>, Error>;
    async fn get_by_id(&self, id: i32) -> Result<Option<Product>, Error>;
    async fn insert(&self, data: ProductInsertable) -> Result<Product, Error>;
    async fn delete_by_id(&self, id: i32) -> Result<(), Error>;
    async fn add_asset(&self, product_id: i32, asset_filename: &String) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct RepoImpl {
    db_pool: Pool,
}

impl RepoImpl {
    pub fn new(db_pool: Pool) -> Self {
        RepoImpl { db_pool }
    }

    async fn get_product_assets<'a>(
        &self,
        product_id: i32,
        transaction: &Transaction<'a>,
    ) -> Result<Vec<Asset>, Error> {
        let assets_rows = transaction
            .query("SELECT * FROM assets WHERE product_id = $1", &[&product_id])
            .await?;

        Ok(assets_rows
            .iter()
            .map(|row| Ok(Asset::from_row_ref(row)?))
            .collect::<Result<Vec<Asset>, Error>>()?)
    }
}

#[async_trait]
impl Repo for RepoImpl {
    async fn get_all(&self) -> Result<Vec<Product>, Error> {
        let mut conn = self.db_pool.get().await?;
        let transaction = conn.transaction().await?;

        let result = async {
            let product_rows = transaction.query("SELECT * FROM products", &[]).await?;
            let transaction_ref = &transaction;

            Ok::<_, Error>(
                product_rows
                    .iter()
                    .map(|row| async move {
                        let mut product = Product::try_from(row)?;
                        product.assets =
                            self.get_product_assets(product.id, transaction_ref).await?;

                        Ok::<_, Error>(product)
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

    async fn get_by_id(&self, id: i32) -> Result<Option<Product>, Error> {
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

        match result {
            Ok(_) => transaction.commit().await?,
            Err(_) => transaction.rollback().await?,
        };

        result
    }

    async fn insert(&self, product: ProductInsertable) -> Result<Product, Error> {
        let conn = self.db_pool.get().await?;

        let row = conn
            .query_one(
                "INSERT INTO products (name, price) VALUES ($1, $2) RETURNING *",
                &[&product.name, &product.price],
            )
            .await?;

        Ok(Product::try_from(&row)?)
    }

    async fn delete_by_id(&self, id: i32) -> Result<(), Error> {
        let conn = self.db_pool.get().await?;

        conn.execute("DELETE FROM products WHERE id = $1", &[&id])
            .await?;

        Ok(())
    }

    async fn add_asset(&self, product_id: i32, asset_filename: &String) -> Result<(), Error> {
        let conn = self.db_pool.get().await?;

        conn.execute(
            "INSERT INTO assets (product_id, filename) VALUES ($1, $2)",
            &[&product_id, &asset_filename],
        )
        .await?;

        Ok(())
    }
}
