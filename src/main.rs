#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

extern crate dotenv;

mod product;
mod schema;

use rocket::{Build, Rocket};

#[launch]
fn rocket() -> Rocket<Build> {
    let connection = rustmerce::establish_connection();
    let products_dao = product::ProductImpl::new(connection);

    rocket::build()
        .attach(product::stage())
        .manage(products_dao)
}
