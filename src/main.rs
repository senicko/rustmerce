use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use dotenv::dotenv;
use std::env;
use tokio_postgres::NoTls;

mod product;
mod storage;

fn init_db_pool() -> Pool {
    let db_username = env::var("DB_USERNAME").expect("DB_USERNAME isn't set");
    let db_url = env::var("DB_NAME").expect("DB_NAME isn't set");

    let mut config = Config::new();

    config.user = Some(db_username);
    config.dbname = Some(db_url);
    config.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    config
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("failed to initialize pool")
}

fn init_logger() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    init_logger();

    std::fs::create_dir_all("./assets").expect("Failed to create ./assets");

    let db_pool = init_db_pool();
    let storage_service = storage::Storage::new();
    let product_store = product::store::ProductStore::new(db_pool);

    HttpServer::new(move || {
        let logger = Logger::default();

        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .wrap(logger)
            .app_data(web::Data::new(product_store.clone()))
            .app_data(web::Data::new(storage_service.clone()))
            .service(actix_files::Files::new("/assets", "./assets").show_files_listing())
            .configure(product::handlers::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
