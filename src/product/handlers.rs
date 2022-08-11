use crate::{
    product::{
        repo::{Repo, RepoImpl},
        service::{Service, ServiceError, ServiceImpl},
        ProductInsertable,
    },
    storage::{Storage, StorageImpl},
};
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures::StreamExt;
use serde_json::json;

async fn list_products(
    product_service: web::Data<ServiceImpl>,
) -> Result<HttpResponse, ServiceError> {
    let products = product_service.get_all().await?;

    Ok(HttpResponse::Ok().json(products))
}

async fn get_product(
    id: web::Path<i32>,
    product_service: web::Data<ServiceImpl>,
) -> Result<HttpResponse, ServiceError> {
    let product = product_service.get_one(id.into_inner()).await?;

    match product {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

async fn create_product(
    data: web::Json<ProductInsertable>,
    product_service: web::Data<ServiceImpl>,
) -> Result<HttpResponse, ServiceError> {
    let created = product_service.create(data.into_inner()).await?;

    Ok(HttpResponse::Created().json(created))
}

async fn delete_product(
    id: web::Path<i32>,
    product_service: web::Data<ServiceImpl>,
) -> Result<HttpResponse, ServiceError> {
    product_service.delete(id.into_inner()).await?;

    Ok(HttpResponse::Ok().finish())
}

// async fn add_product_asset(
//     id: web::Path<i32>,
//     mut payload: Multipart,
//     product_repo: web::Data<RepoImpl>,
//     storage_service: web::Data<StorageImpl>,
// ) -> Result<HttpResponse, Error> {
//     while let Some(Ok(field)) = payload.next().await {
//         let content_disposition = field.content_disposition();

//         if Some("payload") == content_disposition.get_name() {
//             let filename = storage_service.save_image(field).await?;
//             let result = product_repo.add_asset(id.into_inner(), &filename).await;

//             return match result {
//                 Ok(()) => Ok(HttpResponse::Created().json(json!({ "filename": filename }))),
//                 Err(e) => {
//                     storage_service.delete_image(&filename).await?;

//                     Err(e)
//                 }
//             };
//         }
//     }

//     Ok(HttpResponse::BadRequest().json(json!({
//         "message": "payload field missing"
//     })))
// }

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .service(
                web::resource("")
                    .route(web::get().to(list_products))
                    .route(web::post().to(create_product)),
            )
            .service(
                web::scope("{id}").service(
                    web::resource("")
                        .route(web::get().to(get_product))
                        .route(web::delete().to(delete_product)),
                ), // .route("/assets", web::post().to(add_product_asset)),
            ),
    );
}
