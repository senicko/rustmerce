#[macro_use] extern crate rocket;

use rocket::serde::{Deserialize};
use rocket::serde::json::{Json};

#[derive(Deserialize)]
struct Product<'r> {
    name: &'r str,
    price: u32
}

#[post("/", data = "<product>")]
fn create_product(product: Json<Product<'_>>) {
    println!("product name={} price={}", product.name, product.price)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/product", routes![create_product])
}

