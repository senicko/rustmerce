// #[macro_use]
// extern crate rocket;

// mod product;
// use rocket::{Build, Rocket};

// #[launch]
// fn rocket() -> Rocket<Build> {
//     rocket::build().attach(product::stage())
// }

#[macro_use]
extern crate diesel;
extern crate dotenv;

mod models;
mod schema;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use crate::models::Product;

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url)
        .expect(format!("Error connecting to {}", database_url).as_str())
}

fn main() {
    use crate::schema::products::dsl::{name, products};

    let connection = establish_connection();

    let results = products
        .filter(name.eq("Shoes"))
        .limit(5)
        .load::<Product>(&connection)
        .expect("Error loading products");

    for product in results {
        println!("name: {}, price: {}", product.name, product.price)
    }
}
