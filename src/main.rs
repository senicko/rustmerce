use std::sync::Mutex;

use actix_web::{get, web, App, HttpServer};

struct AppState {
    app_name: String,
}

#[get("/welcome")]
async fn hello(data: web::Data<AppState>) -> String {
    let app_name = &data.app_name;
    format!("Welcome to {app_name}!")
}

struct AppStateWithCounter {
    counter: Mutex<i32>,
}

async fn index(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;

    format!("Request number: {counter}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
            }))
            .route("/", web::get().to(index))
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
