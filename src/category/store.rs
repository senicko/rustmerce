use deadpool_postgres::Pool;

use super::Category;

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

    // group_categories groups categories together.
    // TODO: There is a lot of cloning. Maybe there is a way to avoid it?
    fn group_categories(&self, id: Option<i32>, categories: Vec<Category>) -> Vec<Category> {
        let mut parent_categories = categories
            .iter()
            .cloned()
            .filter(|c| c.parent_id == id)
            .collect::<Vec<Category>>();

        parent_categories.iter_mut().for_each(|c| {
            let children = self.group_categories(Some(c.id), categories.clone());
            c.children = children;
        });

        parent_categories
    }

    pub async fn get_all(&self) -> Result<Vec<Category>, CategoryStoreError> {
        let conn = self.db_pool.get().await?;

        let rows = conn.query("SELECT * FROM categories", &[]).await?;

        let all_categories = rows
            .iter()
            .map(|row| Ok(Category::try_from(row)?))
            .collect::<Result<Vec<Category>, CategoryStoreError>>()?;

        Ok(self.group_categories(None, all_categories))
    }

    // WIP
    // pub async fn get_one(&self, id: i32) -> Result<Category, CategoryStoreError> {
    //     let conn = self.db_pool.get().await?;

    //     let rows = conn
    //         .query(
    //             "SELECT c.* FROM categories as c, get_category_tree(1) as t WHERE c.id IN (t)",
    //             &[&id],
    //         )
    //         .await?;

    //     let category =
    //         Category::try_from(rows.get(0).ok_or(CategoryStoreError::MappingFailed(()))?)?;

    //     let mut category = Category::try_from(row)?;
    //     category.children = self.group_categories(id).await?;

    //     Ok(category)
    // }
}
