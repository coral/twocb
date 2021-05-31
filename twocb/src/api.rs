#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
use crate::data;
use std::net::SocketAddr;

pub struct API {
    db: data::DataLayer,
}

impl API {
    pub fn new(db: data::DataLayer) -> API {
        API { db }
    }

    pub async fn start(&'static mut self, addr: SocketAddr) {
        rocket::ignite().mount("/", routes![index]).launch();
        // let routes = warp::any().map(|| {
        //     let mut resp = String::new();
        //     for (key, value) in self.db.get_states() {
        //         resp.push_str(&format!("{}: {} \n", &key, &value));
        //     }
        //     return resp;
        // });

        // warp::serve(routes).run(addr).await;
    }
}
