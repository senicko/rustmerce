use actix_multipart::Multipart;
use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse, ResponseError};
use anyhow::Context;
use serde_json::json;
use validator::Validate;

use super::store::ProductStore;
use crate::{product::ProductInsertable, storage::Storage};

#[derive(thiserror::Error, Debug)]
enum ProductApiError {
    #[error("Bad request")]
    BadRequest(String),

    #[error("Validation failed")]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl ResponseError for ProductApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::ValidationError(_) | Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let mut response = HttpResponse::build(self.status_code());

        match self {
            Self::ValidationError(e) => {
                response.json(json!({"message": "Validation failed", "errors": e.errors()}))
            }
            Self::BadRequest(_) => response.json(json!({ "message": "Bad request" })),
            Self::Internal(_) => response.json(json!({ "message": "Internal server error" })),
        }
    }
}

async fn list_products(
    product_store: web::Data<ProductStore>,
) -> Result<HttpResponse, ProductApiError> {
    let products = product_store
        .get_all()
        .await
        .context("Failed to get products")?;

    Ok(HttpResponse::Ok().json(products))
}

async fn get_product(
    id: web::Path<i32>,
    product_store: web::Data<ProductStore>,
) -> Result<HttpResponse, ProductApiError> {
    let product = product_store
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
    product_store: web::Data<ProductStore>,
) -> Result<HttpResponse, ProductApiError> {
    data.validate()?;

    let created = product_store
        .insert(data.into_inner())
        .await
        .context("Failed to create product")?;

    Ok(HttpResponse::Created().json(created))
}

async fn delete_product(
    id: web::Path<i32>,
    product_store: web::Data<ProductStore>,
) -> Result<HttpResponse, ProductApiError> {
    product_store
        .delete(id.into_inner())
        .await
        .context("Failed to delete product")?;

    Ok(HttpResponse::Ok().finish())
}

async fn add_product_asset(
    req: HttpRequest,
    id: web::Path<i32>,
    multipart: Multipart,
    storage: web::Data<Storage>,
    product_store: web::Data<ProductStore>,
) -> Result<HttpResponse, ProductApiError> {
    // check if content_length isn't too large

    if let Some(conent_length) = req.headers().get("content-length") {
        if conent_length
            .to_str()
            .context("Failed to parse content-length to str")?
            .parse::<u64>()
            .context("Failed to parse content-length to u64")?
            // 2 MB
            > 1024 * 1024 * 2
        {
            return Err(ProductApiError::BadRequest(
                "File can't be bigger than 1MB".to_string(),
            ));
        }
    } else {
        return Err(ProductApiError::BadRequest(
            "Missing content-length header".to_string(),
        ));
    }

    // save uploaded file

    let asset_filename = storage
        .save_image(multipart)
        .await
        .context("Failed to save image")?;

    match product_store
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
