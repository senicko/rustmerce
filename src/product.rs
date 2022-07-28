use actix_web::{delete, get, http::header::ContentType, post, web, HttpResponse};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "products")]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
}

#[derive(Serialize, Deserialize)]
pub struct ProductInsertable {
    pub name: String,
    pub price: f64,
}

#[get("")]
async fn list_products(db_pool: web::Data<Pool>) -> HttpResponse {
    let conn = db_pool.get().await.unwrap();

    let stmt = conn.prepare_cached("SELECT * FROM products").await.unwrap();
    let rows = conn.query(&stmt, &[]).await.unwrap();

    let products: Vec<Product> = rows
        .iter()
        .map(|r| Product::from_row_ref(r).unwrap())
        .collect();

    let body = serde_json::to_string(&products).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body)
}

#[get("/{id}")]
async fn get_product(id: web::Path<i32>, db_pool: web::Data<Pool>) -> HttpResponse {
    let conn = db_pool.get().await.unwrap();

    let stmt = conn
        .prepare_cached("SELECT * FROM products WHERE id = $1")
        .await
        .unwrap();

    let row = conn.query_one(&stmt, &[&id as &i32]).await;

    match row {
        Ok(r) => {
            let product = Product::from_row(r).unwrap();

            let body = serde_json::to_string(&product).unwrap();

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(body)
        }
        Err(_) => HttpResponse::NotFound().json(json!({
            "message": "Product not found"
        })),
    }
}

#[post("")]
async fn create_product() -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

#[delete("/{id}")]
async fn delete_product(id: web::Path<String>) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .service(list_products)
            .service(get_product)
            .service(create_product)
            .service(delete_product),
    );
}
