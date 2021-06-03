use crate::data;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use std::net::SocketAddr;
use std::sync::Arc;

// #[get("/")]
// async fn index(data: web::Data<data::DataLayer>) -> String {
//     //let app_name = &data.app_name; // <- get app_name

//     //let kek = &data.get_states();
//     //serde_json::to_string(kek).unwrap()
//     format!("Hello")
// }

async fn hello(data: web::Data<Arc<data::DataLayer>>) -> String {
    //let app_name = &data.app_name; // <- get app_name

    let kek = &data.get_states();
    serde_json::to_string(kek).unwrap()
    //format!("Hello !") // <- response with app_name
}

#[actix_web::main]
pub async fn start(state: Arc<data::DataLayer>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .route("/", web::get().to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
