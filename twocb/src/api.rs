use crate::controller;
use crate::data;
use crate::layers;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

async fn hello(data: web::Data<data::DataLayer>) -> String {
    //let app_name = &data.app_name; // <- get app_name
    //&data.write_state("kek", &arr);
    //data.clone().write().unwrap().write_state("kek", &arr);
    let kek = data.get_states();
    serde_json::to_string(&kek).unwrap()
}

#[get("/states")]
async fn get_states(data: web::Data<RwLock<data::DataLayer>>) -> impl Responder {
    let kek = data.read().await.get_states();
    HttpResponse::Ok().body(serde_json::to_string(&kek).unwrap())
}

#[derive(Deserialize)]
struct NewState {
    key: String,
    state: String,
}
#[post("/state")]
async fn set_state(
    info: web::Json<NewState>,
    data: web::Data<RwLock<data::DataLayer>>,
) -> impl Responder {
    data.write()
        .await
        .write_state(&info.key, info.state.as_bytes());
    HttpResponse::Ok().body("hello")
}

#[get("/layers")]
async fn get_layers(ctrl: web::Data<Arc<Mutex<controller::Controller>>>) -> impl Responder {
    let res = ctrl.clone().lock().await.get_links_string().await;
    HttpResponse::Ok().body(res)
}

struct NewLayer {
    key: String,
    state: String,
}

#[post("/layer")]
async fn add_layer(
    info: web::Json<layers::DeLink>,
    ctrl: web::Data<Arc<Mutex<controller::Controller>>>,
) -> impl Responder {
    ctrl.lock().await.add_link(info.0).await;
    HttpResponse::Ok().body("Yesssss")
}

#[actix_web::main]
pub async fn start(
    socket: SocketAddr,
    state: data::DataLayer,
    ctrl: Arc<Mutex<controller::Controller>>,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .data(RwLock::new(state.clone()))
            .data(ctrl.clone())
            .service(get_states)
            .service(set_state)
            .service(get_layers)
            .service(add_layer)
            .route("/", web::get().to(hello))
    })
    .bind(socket)?
    .disable_signals()
    .workers(4)
    .run()
    .await
}
