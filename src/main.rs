use actix_web::{get, http::header::ContentType, web, App, HttpResponse, HttpServer, Responder};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use product::Product;
use std::sync::Mutex;
use tokio_postgres::NoTls;

mod product;

fn initialize_db_pool() -> Pool {
    let mut config = Config::new();

    // TODO: Load configuration from .env
    config.user = Some("sebastianflajszer".to_string());
    config.dbname = Some("rustmerce".to_string());

    config.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    config
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("failed to initialize pool")
}

#[get("/test")]
async fn db_test(db_pool: web::Data<Pool>) -> HttpResponse {
    let conn = db_pool.get().await.unwrap();

    let stmt = conn.prepare_cached("SELECT * FROM products").await.unwrap();
    let rows = conn.query(&stmt, &[]).await.unwrap();

    let id: i32 = rows[0].get(0);
    let name: &str = rows[0].get(1);
    let price: f64 = rows[0].get(2);

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(format!("{} {} {}", id, name, price))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_pool = web::Data::new(initialize_db_pool());

    let product_list = web::Data::new(Mutex::new(vec![Product {
        id: "1".to_string(),
        name: "Basket".to_string(),
        price: 25.0,
    }]));

    HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .app_data(product_list.clone())
            .configure(product::config)
            .service(db_test)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
