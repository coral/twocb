mod audio;
mod data;
mod layers;
mod patterns;
mod pixels;
mod producer;

use log;
use std::env;
use std::sync::Arc;
use tokio::task;

use std::time::{Duration, Instant};

#[tokio::main]
pub async fn main() {
    env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();
    let map = pixels::Mapping::load_from_file("files/mappings/v6.json").unwrap();

    let join = task::spawn(async {
        let mut input = audio::AudioInput::new(48_000, 512, 1);
        let mut audioprocessing = audio::Processing::new();
        audioprocessing.run(&mut input);
    });

    join.await;

    //std::thread::sleep(std::time::Duration::from_secs(10));

    //input.start();

    //data::init();

    // let mut prod = producer::Producer::new(200.0);

    // let run = prod.start();
    // run.await;

    // let mut layer_manager = layers::Manager::new();
    // layer_manager.sm();

    patterns::dynamic::initalize_runtime();

    // let join = task::spawn(async {
    //     let map = pixels::Mapping::load_from_file("files/mappings/v6.json").unwrap();
    //     let mut p = patterns::dynamic::Pattern::create("examples/debug.js", map.clone());
    //     p.load();
    //     p.setup();
    //     p.register();
    //     p.process()
    // });
    let mut p = patterns::dynamic::Pattern::create("examples/debug.js", map.clone());
    p.load();
    p.setup();
    p.register();
    // let result = join.await;
    // dbg!(result);
    // let now = Instant::now();
    // let invocations = 10000;
    // for _ in 0..invocations {
    //     let mut m = p.process();
    //     m[0] = 1.0;
    //     dbg!(m);
    // }
    // println!(
    //     "Time: {}ms, {} invocations per second,",
    //     now.elapsed().as_millis(),
    //     invocations as f64 / (now.elapsed().as_millis() as f64 / 1000.)
    // );
}
