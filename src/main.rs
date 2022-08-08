use actix_web::{middleware::Logger, web, App, HttpServer};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use dotenv::dotenv;
use std::env;
use tokio_postgres::NoTls;

mod error;
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
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    init_logger();

    std::fs::create_dir_all("./assets").expect("Failed to create ./assets");

    let db_pool = init_db_pool();
    let product_repo = product::repo::RepoImpl::new(db_pool.clone());
    let storage_service = storage::StorageImpl;

    HttpServer::new(move || {
        let logger = Logger::default();

        App::new()
            .wrap(logger)
            .app_data(web::Data::new(product_repo.clone()))
            .app_data(web::Data::new(storage_service.clone()))
            .service(actix_files::Files::new("/assets", "./assets").show_files_listing())
            .configure(product::handlers::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
