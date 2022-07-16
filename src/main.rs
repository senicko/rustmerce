#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::State;
use std::sync::atomic::{AtomicUsize, Ordering};

struct HitCount(AtomicUsize);

#[derive(Deserialize)]
struct Product<'r> {
    name: &'r str,
    price: u32,
}

#[post("/", data = "<product>")]
fn create_product(product: Json<Product<'_>>) {
    println!("product name={} price={}", product.name, product.price)
}

#[get("/")]
fn count(hit_count: &State<HitCount>) -> String {
    let count = hit_count.0.fetch_add(1, Ordering::Relaxed) + 1;
    format!("Number of visits {}", count)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/products", routes![create_product])
        .mount("/count", routes![count])
        .manage(HitCount(AtomicUsize::new(0)))
}
