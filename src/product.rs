use crate::error::AppError;
use actix_web::{delete, get, post, web, HttpResponse};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "products")]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductInsertable {
    pub name: String,
    pub price: f64,
}

#[get("")]
async fn list_products(db_pool: web::Data<Pool>) -> Result<HttpResponse, AppError> {
    let conn = db_pool.get().await?;

    let stmt = conn.prepare_cached("SELECT * FROM products").await?;
    let rows = conn.query(&stmt, &[]).await?;

    let products = rows
        .iter()
        .map(|r| Ok(Product::from_row_ref(r)?))
        .collect::<Result<Vec<Product>, AppError>>()?;

    Ok(HttpResponse::Ok().json(products))
}

#[get("/{id}")]
async fn get_product(
    id: web::Path<i32>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, AppError> {
    let conn = db_pool.get().await?;

    let stmt = conn
        .prepare_cached("SELECT * FROM products WHERE id = $1")
        .await?;
    let row = conn.query_opt(&stmt, &[&id as &i32]).await?;

    match row {
        Some(r) => {
            let product = Product::from_row(r)?;

            Ok(HttpResponse::Ok().json(product))
        }
        // TODO: Is Not Found really an error in this situation? It is more like a normal response.
        // Maybe this can be abstracted in a better way because currently logger prints it like an error :/.
        None => Err(AppError {
            cause: None,
            message: Some("Product not found".to_string()),
            error_type: crate::error::AppErrorType::NotFound,
        }),
    }
}

#[post("")]
async fn create_product(
    product: web::Json<ProductInsertable>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, AppError> {
    let conn = db_pool.get().await?;

    let stmt = conn
        .prepare_cached("INSERT INTO products (name, price) VALUES ($1, $2) RETURNING *")
        .await?;
    let row = conn
        .query_one(&stmt, &[&product.name, &product.price])
        .await?;

    let product = Product::from_row(row)?;

    Ok(HttpResponse::Created().json(product))
}

#[delete("/{id}")]
async fn delete_product(
    id: web::Path<i32>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, AppError> {
    let conn = db_pool.get().await?;

    let stmt = conn
        .prepare_cached("DELETE FROM products WHERE id = $1")
        .await?;

    // TODO: Should number of modified rows be checked?
    conn.execute(&stmt, &[&id as &i32]).await?;

    Ok(HttpResponse::Ok().finish())
}
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .service(list_products)
            .service(get_product)
            .service(create_product)
            .service(delete_product),
    );
}
