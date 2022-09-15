use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use dotenv::dotenv;
use redis::Commands;
use std::env;
use tokio_postgres::NoTls;

extern crate redis;

mod category;
mod product;
mod storage;

fn init_redis_connection() -> redis::Connection {
    let client = redis::Client::open("redis://127.0.0.1/").expect("Invalid redis connection url");

    client
        .get_connection()
        .expect("Failed to connect with redis")
}

fn test_redis() {
    let mut redis_conn = init_redis_connection();

    // throw away the result, just make sure it does not fail
    let _: () = redis_conn.set("my_key", 42).unwrap();

    // read back the key and return it.  Because the return value
    // from the function is a result for integer this will automatically
    // convert into one.
    let value: i32 = redis_conn.get("my_key").unwrap();

    println!("{value}");
}

fn init_db_pool() -> Pool {
    let db_username = env::var("DB_USERNAME").expect("DB_USERNAME enviroment variable missing");
    let db_url = env::var("DB_NAME").expect("DB_NAME enviroment variable missing");

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

    test_redis();

    let product_store = product::store::ProductStore::new(db_pool.clone());
    let category_store = category::store::CategoryStore::new(db_pool.clone());

    let storage_service = storage::Storage::new();

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
            .app_data(web::Data::new(category_store.clone()))
            .service(actix_files::Files::new("/assets", "./assets").show_files_listing())
            .configure(product::handlers::config)
            .configure(category::handlers::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
