use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::Row;

pub mod handlers;
pub mod repo;
pub mod service;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "assets")]
pub struct Asset {
    pub id: i32,
    pub filename: String,
}

#[derive(Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub assets: Vec<Asset>,
}

impl TryFrom<&Row> for Product {
    type Error = tokio_pg_mapper::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Product {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            price: row.try_get("price")?,
            assets: Vec::new(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductInsertable {
    pub name: String,
    pub price: f64,
}
