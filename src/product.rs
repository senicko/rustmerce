use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::Row;
use validator::Validate;

pub mod handlers;
pub mod store;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "assets")]
pub struct Asset {
    pub id: i32,
    pub filename: String,
}

#[derive(Serialize, Deserialize)]
pub enum ProductStatus {
    Published,
    Draft,
}

#[derive(Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub status: ProductStatus,
    pub assets: Vec<Asset>,
}

impl TryFrom<&Row> for Product {
    type Error = tokio_pg_mapper::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let status: &str = row.try_get("status")?;

        Ok(Product {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            price: row.try_get("price")?,
            status: {
                match status {
                    "Draft" => ProductStatus::Draft,
                    _ => ProductStatus::Published,
                }
            },
            assets: Vec::new(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ProductInsertable {
    #[validate(length(min = 1))]
    pub name: String,

    #[validate(range(min = 1))]
    pub price: f64,
}
