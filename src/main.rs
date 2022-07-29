use actix_web::{middleware::Logger, web, App, HttpServer};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use dotenv::dotenv;
use std::env;
use tokio_postgres::NoTls;

mod error;
mod product;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let db_pool = web::Data::new(init_db_pool());

    HttpServer::new(move || {
        let logger = Logger::default();

        App::new()
            .wrap(logger)
            .app_data(db_pool.clone())
            .configure(product::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
