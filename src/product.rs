use crate::error::AppError;
use actix_web::{delete, get, http::header::ContentType, post, web, App, HttpResponse};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "products")]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
}

#[derive(Serialize, Deserialize)]
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
        .map(|r| {
            // TODO: Try to simplify this later
            let product = Product::from_row_ref(r);

            match product {
                Ok(p) => Ok(p),
                Err(e) => Err(AppError::from(e)),
            }
        })
        .collect::<Result<Vec<Product>, AppError>>()?;

    let body = serde_json::to_string(&products).unwrap();

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body))
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
    let row = conn.query_one(&stmt, &[&id as &i32]).await?;

    let product = Product::from_row(row)?;
    let body = serde_json::to_string(&product)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body))
}

#[post("")]
async fn create_product() -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

#[delete("/{id}")]
async fn delete_product(id: web::Path<String>) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
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
