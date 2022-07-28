use actix_web::{web, App, HttpServer};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_pool = web::Data::new(initialize_db_pool());

    HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .configure(product::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
