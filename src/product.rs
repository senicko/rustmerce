use crate::{
    error::AppError,
    product::repo::{Repo, RepoImpl},
    storage::{Storage, StorageImpl},
};
use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, HttpResponse};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_pg_mapper_derive::PostgresMapper;

pub mod repo;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "assets")]
pub struct Asset {
    pub id: i32,
    pub filename: String,
}

// TODO: Implement From<ProductRaw> for Product?

#[derive(Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub assets: Option<Vec<Asset>>,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "products")]
pub struct ProductRaw {
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
    product_repo: web::Data<RepoImpl>,
    storage_service: web::Data<StorageImpl>,
) -> Result<HttpResponse, AppError> {
    // TODO: Create guard or sth that checks if the product exists
    let mut filename: Option<String> = None;

    while let Some(field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();

        if Some("image") == content_disposition.get_name() {
            filename = Some(storage_service.save_image(field).await?);
            break;
        }
    }

    match filename {
        Some(filename) => {
            product_repo.add_asset(id.into_inner(), &filename).await?;

            Ok(HttpResponse::Created().json(json!({ "filename": filename })))
        }
        None => Err(AppError {
            cause: Some("image field missing".to_string()),
            message: Some("Invalid request".to_string()),
            error_type: crate::error::AppErrorType::Internal,
        }),
    }
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
