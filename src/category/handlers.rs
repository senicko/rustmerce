use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use serde_json::json;

use super::store::CategoryStore;

// TODO: This can be somehow be generalized. Product handlers have Internal error too.
#[derive(thiserror::Error, Debug)]
pub enum CategoryApiError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl ResponseError for CategoryApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let mut response = HttpResponse::build(self.status_code());

        match self {
            Self::Internal(_) => response.json(json!({ "message": "Internal server error" })),
        }
    }
}

async fn list_categories(
    category_store: web::Data<CategoryStore>,
) -> Result<HttpResponse, CategoryApiError> {
    let categories = category_store
        .get_all()
        .await
        .context("Failed to get categories")?;

    Ok(HttpResponse::Ok().json(categories))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/categories").route("", web::get().to(list_categories)));
}
