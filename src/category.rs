use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

pub mod handlers;
pub mod store;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "categories")]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
}
