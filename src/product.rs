use actix_web::{delete, get, http::header::ContentType, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct Product {
    pub name: String,
    pub price: f32,
}

pub type ProductList = Mutex<Vec<Product>>;

#[get("")]
async fn list_products(product_list: web::Data<ProductList>) -> impl Responder {
    let lock = product_list.lock().unwrap();

    // TODO: Learn about rust's Mutex (Idk if this is the correct way of doing that)
    let body = serde_json::to_string(lock.deref()).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body)
}

#[get("/{id}")]
async fn get_product(id: web::Path<String>) -> String {
    format!("Getting a product by its id (id={id})")
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
