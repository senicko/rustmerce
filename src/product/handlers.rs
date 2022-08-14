use crate::{
    product::{service::ProductService, ProductInsertable},
    storage::Storage,
};
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;

#[derive(thiserror::Error, Debug)]
enum ProductApiError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl ResponseError for ProductApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ProductApiError::Internal(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn list_products(
    product_service: web::Data<ProductService>,
) -> Result<HttpResponse, ProductApiError> {
    let products = product_service
        .get_all()
        .await
        .context("Failed to get products")?;

    Ok(HttpResponse::Ok().json(products))
}

async fn get_product(
    id: web::Path<i32>,
    product_service: web::Data<ProductService>,
) -> Result<HttpResponse, ProductApiError> {
    let product = product_service
        .get_one(id.into_inner())
        .await
        .context("Failed to get product")?;

    match product {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

async fn create_product(
    data: web::Json<ProductInsertable>,
    product_service: web::Data<ProductService>,
) -> Result<HttpResponse, ProductApiError> {
    let created = product_service
        .create(data.into_inner())
        .await
        .context("Failed to create product")?;

    Ok(HttpResponse::Created().json(created))
}

async fn delete_product(
    id: web::Path<i32>,
    product_service: web::Data<ProductService>,
) -> Result<HttpResponse, ProductApiError> {
    product_service
        .delete(id.into_inner())
        .await
        .context("Failed to delete product")?;

    Ok(HttpResponse::Ok().finish())
}

async fn add_product_asset(
    id: web::Path<i32>,
    multipart: Multipart,
    storage: web::Data<Storage>,
    product_service: web::Data<ProductService>,
) -> Result<HttpResponse, ProductApiError> {
    let asset_filename = storage
        .save_image(multipart)
        .await
        .context("Failed to save image")?;

    match product_service
        .add_asset(id.to_owned(), &asset_filename)
        .await
        .context("Failed to add asset")
    {
        Ok(asset) => Ok(HttpResponse::Created().json(asset)),
        Err(e) => {
            storage
                .delete_image(&asset_filename)
                .await
                .expect("Failed to remove the file");

            Err(ProductApiError::Internal(e))
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .service(
                web::resource("")
                    .route(web::get().to(list_products))
                    .route(web::post().to(create_product)),
            )
            .service(
                web::scope("{id}")
                    .service(
                        web::resource("")
                            .route(web::get().to(get_product))
                            .route(web::delete().to(delete_product)),
                    )
                    .route("/assets", web::post().to(add_product_asset)),
            ),
    );
}
