use crate::schema::products;
use diesel::RunQueryDsl;
use rocket::fairing::AdHoc;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{routes, State};
use rustmerce::DbPool;

#[derive(Queryable, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub price: f64,
}

pub trait ProductDao {
    fn get_all(&self) -> Vec<Product>;
}

pub struct ProductImpl {
    db_pool: DbPool,
}

impl ProductImpl {
    pub fn new(db_pool: DbPool) -> ProductImpl {
        ProductImpl { db_pool }
    }
}

impl ProductDao for ProductImpl {
    fn get_all(&self) -> Vec<Product> {
        let connection = self.db_pool.get().unwrap();
        products::table.load::<Product>(&connection).unwrap()
    }
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
fn get_all(products_dao: &State<ProductImpl>) -> Json<Vec<Product>> {
    Json(products_dao.get_all())
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Product", |rocket| async {
        rocket.mount("/products", routes![get_all])
    })
}

#[cfg(test)]
mod test {
    // use rocket::local::blocking::Client;

    // #[test]
    // fn get_all() {
    //     let client = Client::tracked(rocket::build().attach(super::stage())).unwrap();

    //     let response = client.get("/").dispatch();

    //     assert_eq!(0, 0);
    // }
}
