use crate::data;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use std::net::SocketAddr;
use std::sync::Arc;

async fn hello(data: web::Data<Arc<data::DataLayer>>) -> String {
    //let app_name = &data.app_name; // <- get app_name

    let kek = &data.get_states();
    serde_json::to_string(kek).unwrap()
    //format!("Hello !") // <- response with app_name
}

#[actix_web::main]
pub async fn start(socket: SocketAddr, state: Arc<data::DataLayer>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .route("/", web::get().to(hello))
    })
    .bind(socket)?
    .disable_signals()
    .workers(4)
    .run()
    .await
}
