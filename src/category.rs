use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

pub mod handlers;
pub mod store;

#[derive(Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub children: Vec<Category>,
}

impl TryFrom<&Row> for Category {
    type Error = tokio_pg_mapper::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Category {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            parent_id: row.try_get("parent_id")?,
            children: Vec::new(),
        })
    }
}
