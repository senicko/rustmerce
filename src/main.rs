#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

use rocket::serde::json::serde_json::json;
use rocket::serde::json::{Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::State;

#[derive(Serialize, Deserialize, Clone)]
struct Product {
    id: Option<usize>,
    name: String,
    price: u32,
}

type ProductList = Mutex<Vec<Product>>;

#[get("/", format = "json")]
async fn get_products<'a>(product_list: &State<ProductList>) -> Option<Json<Vec<Product>>> {
    let lock = product_list.lock().await;
    Some(Json(lock.to_vec()))
}

#[post("/", data = "<product>")]
async fn new_product(product: Json<Product>, product_list: &State<ProductList>) -> Value {
    let mut lock = product_list.lock().await;
    let id = lock.len();

    lock.push(Product {
        id: Some(id),
        ..product.0
    });

    json!({ "id": id })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/products", routes![get_products, new_product])
        .manage(ProductList::new(vec![]))
}
