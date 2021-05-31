use crate::data;
use std::net::SocketAddr;
use warp::Filter;

pub struct API {
    db: data::DataLayer,
}

impl API {
    pub fn new(db: data::DataLayer) -> API {
        API { db }
    }

    pub async fn start(&'static mut self, addr: SocketAddr) {
        let routes = warp::any().map(|| {
            let mut resp = String::new();
            for (key, value) in self.db.get_states() {
                resp.push_str(&format!("{}: {} \n", &key, &value));
            }
            return resp;
        });

        warp::serve(routes).run(addr).await;
    }
}
