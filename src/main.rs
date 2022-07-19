#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

use rocket::response::status;
use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::State;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
struct Product {
    id: Option<String>,
    name: String,
    price: u32,
}

type ProductList = Mutex<Vec<Product>>;

#[get("/", format = "json")]
async fn get_products<'a>(product_list: &State<ProductList>) -> Option<Json<Vec<Product>>> {
    let lock = product_list.lock().await;
    Some(Json(lock.to_vec()))
}

#[get("/<product_id>")]
async fn get_product_by_id(
    product_id: &str,
    product_list: &State<ProductList>,
) -> Result<Json<Product>, status::NotFound<Value>> {
    let lock = product_list.lock().await;

    let product = lock
        .iter()
        .find(|p| p.id.as_deref() == Some(product_id.to_string().as_ref()));

    match product {
        Some(p) => Ok(Json(p.clone())),
        None => Err(status::NotFound(json!({"message": "Product not found"}))),
    }
}

#[post("/", data = "<product>")]
async fn new_product(
    product: Json<Product>,
    product_list: &State<ProductList>,
) -> status::Created<String> {
    let mut lock = product_list.lock().await;

    let id = Uuid::new_v4().to_string();
    let resource_url = format!("http://localhost:8000/products/{}", id);

    lock.push(Product {
        id: Some(id),
        ..product.0
    });

    status::Created::new(resource_url)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/products",
            routes![get_products, get_product_by_id, new_product],
        )
        .manage(ProductList::new(vec![]))
}
