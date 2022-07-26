use actix_web::{web::Data, App, HttpServer};
use product::Product;
use std::sync::Mutex;

mod product;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let product_list = Data::new(Mutex::new(vec![Product {
        name: String::from("Basket"),
        price: 25.0,
    }]));

    HttpServer::new(move || {
        App::new()
            .app_data(product_list.clone())
            .configure(product::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
