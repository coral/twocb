mod data;
mod layers;
mod patterns;
mod pixels;
use std::thread;

use std::time::{Duration, Instant};

fn main() {
    println!("Hello, world!");
    let map = pixels::Mapping::load_from_file("files/mappings/v6.json").unwrap();
    //data::init();

    let mut layer_manager = layers::Manager::new();
    layer_manager.sm();

    patterns::dynamic::initalize_runtime();
    // let mut p = patterns::dynamic::Pattern::create("examples/fn2.js", map.clone());
    // p.load();
    // p.setup();
    // p.register();

    // p.process();
    let mut p = patterns::dynamic::Pattern::create("examples/fn2.js", map.clone());
    p.load();
    p.setup();
    p.register();
    let now = Instant::now();
    for _ in 0..10000000 {
        p.process();
    }
    println!("{}", now.elapsed().as_millis());
    // let t2 = thread::spawn(|| {
    //     let mut p = patterns::dynamic::Pattern::create("examples/fn2.js");
    //     p.load();
    //     let now = Instant::now();
    //     for _ in 0..10000000 {
    //         p.process();
    //     }
    //     println!("{}", now.elapsed().as_millis());
    // });

    // t2.join().unwrap();

    // patterns::dynamic::shutdown_runtime();
}
