#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

extern crate dotenv;

mod product;
mod schema;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use rocket::{Build, Rocket};
use std::env;

type DbPoll = Pool<ConnectionManager<PgConnection>>;

fn establish_connection() -> DbPoll {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder().build(manager).unwrap()
}

#[launch]
fn rocket() -> Rocket<Build> {
    let connection = establish_connection();
    rocket::build().attach(product::stage()).manage(connection)
}

// fn main() {
//     use crate::schema::products::dsl::{name, products};

//     let connection = establish_connection();

//     let results = products
//         .filter(name.eq("Shoes"))
//         .limit(5)
//         .load::<Product>(&connection)
//         .expect("Error loading products");

//     for product in results {
//         println!("name: {}, price: {}", product.name, product.price)
//     }
// }
