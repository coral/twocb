use crate::controller;
use crate::data;
use crate::engines::params::ParamValue;
use crate::layers;
use crate::rtc::{PeerManager, PixelBroadcast};
use crate::ws;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, delete, get, post, put, web};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, broadcast};

// ── Existing endpoints ──────────────────────────────────────────────────

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

// ── New endpoints ───────────────────────────────────────────────────────

/// List JS pattern files in files/dynamic/
#[get("/patterns/files")]
async fn list_pattern_files() -> impl Responder {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir("files/dynamic") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("js") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    files.push(name.to_string());
                }
            }
        }
    }
    files.sort();
    HttpResponse::Ok().json(files)
}

#[derive(Deserialize)]
struct FileInfo {
    name: String,
}

/// Read pattern source
#[get("/patterns/files/{name}")]
async fn get_pattern_file(info: web::Path<FileInfo>) -> impl Responder {
    let path = std::path::Path::new("files/dynamic").join(&info.name);
    if !path.extension().and_then(|e| e.to_str()).is_some_and(|e| e == "js") {
        return HttpResponse::BadRequest().body("Only .js files allowed");
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => HttpResponse::Ok()
            .content_type("text/javascript")
            .body(content),
        Err(_) => HttpResponse::NotFound().body("Pattern not found"),
    }
}

/// Write pattern source (hot-reload via file watcher)
#[put("/patterns/files/{name}")]
async fn put_pattern_file(info: web::Path<FileInfo>, body: web::Bytes) -> impl Responder {
    let path = std::path::Path::new("files/dynamic").join(&info.name);
    if !path.extension().and_then(|e| e.to_str()).is_some_and(|e| e == "js") {
        return HttpResponse::BadRequest().body("Only .js files allowed");
    }
    match std::fs::write(&path, &body) {
        Ok(_) => HttpResponse::Ok().body("saved"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Write failed: {}", e)),
    }
}

/// Create new pattern from template
#[post("/patterns/files")]
async fn create_pattern_file(body: web::Json<serde_json::Value>) -> impl Responder {
    let name = match body.get("name").and_then(|n| n.as_str()) {
        Some(n) => n,
        None => return HttpResponse::BadRequest().body("Missing 'name' field"),
    };
    if !name.ends_with(".js") {
        return HttpResponse::BadRequest().body("Name must end with .js");
    }
    let path = std::path::Path::new("files/dynamic").join(name);
    if path.exists() {
        return HttpResponse::Conflict().body("File already exists");
    }
    let template = r#"// New pattern
var pixelCount;

function beforeRender(frame, delta) {
}

function render3D(index, x, y, z) {
  hsv(index, 0, 0, 0);
}
"#;
    match std::fs::write(&path, template) {
        Ok(_) => HttpResponse::Created().body("created"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Create failed: {}", e)),
    }
}

/// Delete pattern file
#[delete("/patterns/files/{name}")]
async fn delete_pattern_file(info: web::Path<FileInfo>) -> impl Responder {
    let path = std::path::Path::new("files/dynamic").join(&info.name);
    if !path.extension().and_then(|e| e.to_str()).is_some_and(|e| e == "js") {
        return HttpResponse::BadRequest().body("Only .js files allowed");
    }
    match std::fs::remove_file(&path) {
        Ok(_) => HttpResponse::Ok().body("deleted"),
        Err(_) => HttpResponse::NotFound().body("File not found"),
    }
}

/// Serve pixel mapping JSON
#[get("/mapping")]
async fn get_mapping(cfg: web::Data<Arc<crate::config::Config>>) -> impl Responder {
    let path = std::path::Path::new("files/mappings").join(&cfg.mapping);
    match std::fs::read_to_string(&path) {
        Ok(content) => HttpResponse::Ok()
            .content_type("application/json")
            .body(content),
        Err(_) => HttpResponse::NotFound().body("Mapping not found"),
    }
}

/// Get runtime config
#[get("/config")]
async fn get_config(cfg: web::Data<Arc<crate::config::Config>>) -> impl Responder {
    HttpResponse::Ok().json(cfg.get_ref().as_ref())
}

// ── Server startup ──────────────────────────────────────────────────────

pub struct ApiState {
    pub db: data::DataLayer,
    pub ctrl: Arc<Mutex<controller::Controller>>,
    pub peer_manager: Arc<Mutex<PeerManager>>,
    pub pixel_broadcast: PixelBroadcast,
    pub state_tx: broadcast::Sender<String>,
    pub config: Arc<crate::config::Config>,
}

#[actix_web::main]
pub async fn start(socket: SocketAddr, api_state: ApiState) -> std::io::Result<()> {
    let db = api_state.db;
    let ctrl = api_state.ctrl;
    let peer_manager = Arc::new(api_state.peer_manager);
    let pixel_broadcast = api_state.pixel_broadcast;
    let state_tx = api_state.state_tx;
    let config = api_state.config;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(RwLock::new(db.clone())))
            .app_data(web::Data::new(ctrl.clone()))
            .app_data(web::Data::new(peer_manager.clone()))
            .app_data(web::Data::new(pixel_broadcast.clone()))
            .app_data(web::Data::new(state_tx.clone()))
            .app_data(web::Data::new(config.clone()))
            // Existing
            .service(get_states)
            .service(set_state)
            .service(get_layers)
            .service(get_layers_order)
            .service(add_layer)
            .service(delete_layer)
            .service(set_opacity)
            .service(get_pattern_params)
            .service(set_pattern_param)
            // New: pattern files
            .service(list_pattern_files)
            .service(get_pattern_file)
            .service(put_pattern_file)
            .service(create_pattern_file)
            .service(delete_pattern_file)
            // New: mapping, config
            .service(get_mapping)
            .service(get_config)
            // WebSocket routes
            .route("/ws/signal", web::get().to(ws::ws_signal))
            .route("/ws/state", web::get().to(ws::ws_state))
    })
    .bind(socket)?
    .disable_signals()
    .workers(4)
    .run()
    .await
}
