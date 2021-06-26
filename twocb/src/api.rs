use crate::controller;
use crate::data;
use crate::layers;
use actix_web::{delete, get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

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
    match data
        .write()
        .await
        .write_state(&info.key, info.state.as_bytes())
    {
        Ok(_) => HttpResponse::Ok().body("hello"),
        Err(e) => HttpResponse::BadRequest().body("JSON ERROR: ".to_string() + &e),
    }
}

#[get("/layers")]
async fn get_layers(ctrl: web::Data<Arc<Mutex<controller::Controller>>>) -> impl Responder {
    let res = ctrl.clone().lock().await.get_links_string().await;
    HttpResponse::Ok().body(res)
}

#[post("/layer")]
async fn add_layer(
    info: web::Json<layers::DeLink>,
    ctrl: web::Data<Arc<Mutex<controller::Controller>>>,
) -> impl Responder {
    ctrl.lock().await.add_link(info.0).await;
    HttpResponse::Ok().body("Yesssss")
}

#[derive(Deserialize)]
struct LayerInfo {
    layer_name: String,
}

#[delete("/layer/{layer_name}")]
async fn delete_layer(
    info: web::Path<LayerInfo>,
    ctrl: web::Data<Arc<Mutex<controller::Controller>>>,
) -> impl Responder {
    match ctrl.lock().await.remove_link(&info.layer_name).await {
        true => HttpResponse::Ok().body("Removed layer"),
        false => HttpResponse::NotFound().body("could not find layer"),
    }
}

#[derive(Deserialize)]
struct NewOpacity {
    key: String,
    opacity: f64,
}
#[post("/opacity")]
async fn set_opacity(
    info: web::Json<NewOpacity>,
    ctrl: web::Data<Arc<Mutex<controller::Controller>>>,
) -> impl Responder {
    ctrl.lock().await.set_opacity(&info.key, info.opacity).await;
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
            .service(delete_layer)
            .service(set_opacity)
    })
    .bind(socket)?
    .disable_signals()
    .workers(4)
    .run()
    .await
}
