use std::net::SocketAddr;
use warp::Filter;

pub struct API {}

impl API {
    pub async fn start(addr: SocketAddr) {
        let routes = warp::any().map(|| "Hello, World!");
        warp::serve(routes).run(addr).await;
    }
}
