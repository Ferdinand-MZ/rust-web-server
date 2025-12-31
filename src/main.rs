mod models;
mod services;
mod routes;

use actix_web::{get, App, HttpResponse, HttpServer, Responder, web::Data};
// actix_web adalah framework web untuk bahasa pemrograman Rust dimana kita dapat membuat server web dengan mudah

// use crate::services::db::Database;
use services::db::Database;

use crate::routes::{booking_route::{create_booking, get_bookings, cancel_booking}, dog_route::create_dog, owner_route::create_owner};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

#[actix_web::main] // menandai fungsi main sebagai fungsi utama untuk aplikasi Actix-web
async fn main() -> std::io::Result<()> {
    let db = Database::init().await;
    let db_data = Data::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .service(hello)
            .service(create_booking)
            .service(create_dog)
            .service(create_owner)
            .service(get_bookings)
            .service(cancel_booking)
    })
    .bind(("localhost", 5001))?
    .run()
    .await
}
