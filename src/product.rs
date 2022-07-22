use crate::schema::products;
use crate::DbPoll;
use diesel::RunQueryDsl;
use rocket::fairing::AdHoc;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{routes, State};

#[derive(Queryable, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub price: f64,
}

#[derive(Insertable)]
#[table_name = "products"]
pub struct InsertableProduct {
    pub name: String,
    pub description: String,
    pub price: f64,
}

// all queries all products from the database
#[get("/")]
fn all(db_pool: &State<DbPoll>) -> Json<Vec<Product>> {
    let connection = db_pool.get().unwrap();
    Json(products::table.load::<Product>(&connection).unwrap())
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Product", |rocket| async {
        rocket.mount("/products", routes![all])
    })
}
