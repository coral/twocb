use crate::data;
use crate::layers;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

async fn hello(data: web::Data<data::DataLayer>) -> String {
    //let app_name = &data.app_name; // <- get app_name

    //let arr: [u8; 4] = [10, 20, 30, 40];
    //&data.write_state("kek", &arr);
    //data.clone().write().unwrap().write_state("kek", &arr);
    let kek = data.get_states();
    serde_json::to_string(&kek).unwrap()

    //format!("Hello !") // <- response with app_name
}

#[get("/states")]
async fn get_states(data: web::Data<data::DataLayer>) -> impl Responder {
    let kek = data.get_states();
    HttpResponse::Ok().body(serde_json::to_string(&kek).unwrap())
}

#[derive(Deserialize)]
struct NewState {
    key: String,
    state: String,
}
#[post("/state")]
async fn set_state(info: web::Json<NewState>, data: web::Data<data::DataLayer>) -> impl Responder {
    println!("Key: {}, Value: {}", info.key, info.state);
    &data.write_state(&info.key, info.state.as_bytes());
    HttpResponse::Ok().body("hello")
}

#[actix_web::main]
pub async fn start(socket: SocketAddr, state: data::DataLayer) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(get_states)
            .service(set_state)
            .route("/", web::get().to(hello))
    })
    .bind(socket)?
    .disable_signals()
    .workers(4)
    .run()
    .await
}
