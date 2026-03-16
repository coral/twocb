use crate::controller;
use crate::data;
use crate::engines::params::ParamValue;
use crate::layers;
use actix_web::{App, HttpResponse, HttpServer, Responder, delete, get, post, web};
use serde::Deserialize;
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

#[get("/layers/order")]
async fn get_layers_order(ctrl: web::Data<Arc<Mutex<controller::Controller>>>) -> impl Responder {
    let res = ctrl.clone().lock().await.lookup_order().await.unwrap();
    let ans = serde_json::to_string(&res).unwrap();
    HttpResponse::Ok().body(ans)
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

#[derive(Deserialize)]
struct PatternInfo {
    name: String,
}

#[get("/patterns/{name}/params")]
async fn get_pattern_params(
    info: web::Path<PatternInfo>,
    ctrl: web::Data<Arc<Mutex<controller::Controller>>>,
) -> impl Responder {
    let params = ctrl.lock().await.get_pattern_params(&info.name).await;
    HttpResponse::Ok().json(params)
}

#[derive(Deserialize)]
struct SetParamInfo {
    name: String,
    param: String,
}

#[post("/patterns/{name}/params/{param}")]
async fn set_pattern_param(
    info: web::Path<SetParamInfo>,
    body: web::Json<ParamValue>,
    ctrl: web::Data<Arc<Mutex<controller::Controller>>>,
) -> impl Responder {
    ctrl.lock()
        .await
        .set_pattern_param(&info.name, &info.param, body.into_inner())
        .await;
    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
pub async fn start(
    socket: SocketAddr,
    state: data::DataLayer,
    ctrl: Arc<Mutex<controller::Controller>>,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(RwLock::new(state.clone())))
            .app_data(web::Data::new(ctrl.clone()))
            .service(get_states)
            .service(set_state)
            .service(get_layers)
            .service(get_layers_order)
            .service(add_layer)
            .service(delete_layer)
            .service(set_opacity)
            .service(get_pattern_params)
            .service(set_pattern_param)
    })
    .bind(socket)?
    .disable_signals()
    .workers(4)
    .run()
    .await
}
