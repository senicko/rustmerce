use crate::error::AppError;
use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, HttpResponse};
use deadpool_postgres::Pool;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::io::Write;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use uuid::Uuid;

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

#[post("/{id}/assets")]
async fn add_product_assets(
    id: web::Path<i32>,
    mut payload: Multipart,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, AppError> {
    // TODO: Craete guard or sth that checks if the product exists
    let conn = pool.get().await?;

    while let Some(mut field) = payload.try_next().await? {
        // Handle file upload
        let content_disposition = field.content_disposition();

        let filename = content_disposition
            .get_filename()
            .map_or_else(|| Uuid::new_v4().to_string(), sanitize_filename::sanitize);
        let file_path = format!("./assets/{filename}");

        let mut f = web::block(|| std::fs::File::create(file_path)).await??;
        while let Some(chunk) = field.try_next().await? {
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
        }

        // Insert asset into database
        let stmt = conn
            .prepare_cached("INSERT INTO assets (url, product_id) VALUES ($1, $2)")
            .await?;

        conn.execute(&stmt, &[&"this_will_be_url".to_string(), &id as &i32])
            .await?;
    }

    Ok(HttpResponse::Created().finish())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .service(list_products)
            .service(get_product)
            .service(create_product)
            .service(delete_product)
            .service(add_product_assets),
    );
}
