use crate::{
    error::{Error, ErrorKind},
    product::{
        repo::{Repo, RepoImpl},
        ProductInsertable,
    },
    storage::{Storage, StorageImpl},
};
use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, HttpResponse};
use futures::TryStreamExt;
use serde_json::json;

#[get("")]
async fn list_products(product_repo: web::Data<RepoImpl>) -> Result<HttpResponse, Error> {
    let products = product_repo.get_all().await?;

    Ok(HttpResponse::Ok().json(products))
}

#[get("/{id}")]
async fn get_product(
    id: web::Path<i32>,
    product_repo: web::Data<RepoImpl>,
) -> Result<HttpResponse, Error> {
    let product = product_repo.get_by_id(id.into_inner()).await?;

    match product {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[post("")]
async fn create_product(
    data: web::Json<ProductInsertable>,
    product_repo: web::Data<RepoImpl>,
) -> Result<HttpResponse, Error> {
    let created = product_repo.insert(data.into_inner()).await?;

    Ok(HttpResponse::Created().json(created))
}

#[delete("/{id}")]
async fn delete_product(
    id: web::Path<i32>,
    product_repo: web::Data<RepoImpl>,
) -> Result<HttpResponse, Error> {
    product_repo.delete_by_id(id.into_inner()).await?;

    Ok(HttpResponse::Ok().finish())
}

#[post("/{id}/assets")]
async fn add_product_assets(
    id: web::Path<i32>,
    mut payload: Multipart,
    product_repo: web::Data<RepoImpl>,
    storage_service: web::Data<StorageImpl>,
) -> Result<HttpResponse, Error> {
    while let Some(field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();

        if Some("image") == content_disposition.get_name() {
            let filename = storage_service.save_image(field).await?;

            let result = product_repo.add_asset(id.into_inner(), &filename).await;

            return match result {
                Ok(_) => Ok(HttpResponse::Created().json(json!({ "filename": filename }))),
                Err(e) => {
                    storage_service.delete_image(&filename).await?;

                    Err(e)
                }
            };
        }
    }

    Err(Error::new(
        "`image` must be set".to_string(),
        ErrorKind::Internal,
    ))
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
