mod data;
mod layers;
mod patterns;
mod pixels;
use std::thread;

use std::time::{Duration, Instant};

fn main() {
    println!("Hello, world!");
    //dbg!(pixels::Mapping::load_from_file("files/mappings/v6.json"));
    //data::init();

    let mut layer_manager = layers::Manager::new();
    layer_manager.sm();

    patterns::dynamic::initalize_runtime();
    let t1 = thread::spawn(|| {
        let mut p = patterns::dynamic::Pattern::create("examples/fn2.js");
        p.load();
        let now = Instant::now();
        for _ in 0..10000000 {
            p.process();
        }
        println!("{}", now.elapsed().as_millis());
    });

    let t2 = thread::spawn(|| {
        let mut p = patterns::dynamic::Pattern::create("examples/fn2.js");
        p.load();
        let now = Instant::now();
        for _ in 0..10000000 {
            p.process();
        }
        println!("{}", now.elapsed().as_millis());
    });

    t1.join().unwrap();
    t2.join().unwrap();
}
