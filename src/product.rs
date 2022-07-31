use crate::{
    error::AppError,
    product::repo::{Repo, RepoImpl},
};
use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, HttpResponse};
use deadpool_postgres::Pool;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Write;
use tokio_pg_mapper_derive::PostgresMapper;
use uuid::Uuid;

pub mod repo;

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
async fn list_products(product_repo: web::Data<RepoImpl>) -> Result<HttpResponse, AppError> {
    let products = product_repo.get_all().await?;
    Ok(HttpResponse::Ok().json(products))
}

#[get("/{id}")]
async fn get_product(
    id: web::Path<i32>,
    product_repo: web::Data<RepoImpl>,
) -> Result<HttpResponse, AppError> {
    let product = product_repo.get_by_id(id.into_inner()).await?;

    match product {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Ok(HttpResponse::NotFound().json(json!({
            "message": "Product not found"
        }))),
    }
}

#[post("")]
async fn create_product(
    data: web::Json<ProductInsertable>,
    product_repo: web::Data<RepoImpl>,
) -> Result<HttpResponse, AppError> {
    let created = product_repo.insert(data.into_inner()).await?;

    Ok(HttpResponse::Created().json(created))
}

#[delete("/{id}")]
async fn delete_product(
    id: web::Path<i32>,
    product_repo: web::Data<RepoImpl>,
) -> Result<HttpResponse, AppError> {
    product_repo.delete_by_id(id.into_inner()).await?;

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
