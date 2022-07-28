use actix_web::{delete, get, http::header::ContentType, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::Mutex;
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

pub type ProductList = Mutex<Vec<Product>>;

#[get("")]
async fn list_products(product_list: web::Data<ProductList>) -> HttpResponse {
    let lock = product_list.lock().unwrap();
    let body = serde_json::to_string(lock.deref()).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body)
}

#[get("/{id}")]
async fn get_product(id: web::Path<String>, product_list: web::Data<ProductList>) -> HttpResponse {
    let lock = product_list.lock().unwrap();
    let product = lock.iter().find(|&p| p.id == 1);

    match product {
        Some(p) => {
            let body = serde_json::to_string(p).unwrap();

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(body)
        }
        None => HttpResponse::NotFound().finish(),
    }
}

#[post("")]
async fn create_product() -> String {
    String::from("Creating a new product")
}

#[delete("/{id}")]
async fn delete_product(id: web::Path<String>) -> String {
    String::from("Deleting a product by its id (id={id})")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .service(list_products)
            .service(get_product),
    );
}
