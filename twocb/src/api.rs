use crate::data;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

async fn hello(data: web::Data<Arc<RwLock<data::DataLayer>>>) -> String {
    //let app_name = &data.app_name; // <- get app_name

    //let arr: [u8; 4] = [10, 20, 30, 40];
    //&data.write_state("kek", &arr);
    //data.clone().write().unwrap().write_state("kek", &arr);
    let kek = data.clone().read().unwrap().get_states();
    serde_json::to_string(&kek).unwrap()

    //format!("Hello !") // <- response with app_name
}

#[actix_web::main]
pub async fn start(socket: SocketAddr, state: Arc<RwLock<data::DataLayer>>) -> std::io::Result<()> {
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
