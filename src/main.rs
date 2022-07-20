#[macro_use]
extern crate rocket;

mod product;
use rocket::{Rocket, Build};

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().attach(product::stage())
}
