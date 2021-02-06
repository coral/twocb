mod audio;
mod data;
mod layers;
mod patterns;
mod pixels;
mod producer;
use std::{thread, time};

use log;
use std::env;
use std::sync::Arc;
use tokio::join;
use tokio::sync::oneshot;
use tokio::task;

use std::time::{Duration, Instant};

#[tokio::main]
pub async fn main() {
    env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();
    let map = pixels::Mapping::load_from_file("files/mappings/v6.json").unwrap();

    let audiosetting = audio::StreamSetting {
        sample_rate: 48_000,
        buffer_size: 1024,
        channels: 1,
    };
    let mut input = audio::Input::new(audiosetting);
    let stream = input.start();

    //Aubio

    let stream_processing = stream.clone();
    let (tempop, tempoc) = oneshot::channel();
    let (onsetp, onsetc) = oneshot::channel();
    let ap = task::spawn(async move {
        let mut audioprocessing = audio::Processing::new(audiosetting, stream_processing);
        tempop.send(audioprocessing.tempo_channel()).unwrap();
        onsetp.send(audioprocessing.onset_channel()).unwrap();
        audioprocessing.run();
    });

    let tempo_channel = tempoc.await.unwrap();
    let onset_channel = onsetc.await.unwrap();

    //Colorchord

    let stream_colorchord = stream.clone();
    let mut colorchord = audio::Colorchord::new(audiosetting, stream_colorchord);
    let colorchord_channel = colorchord.channel();
    let cr = task::spawn(async move {
        colorchord.run();
    });

    //join!(ap, cr);

    //std::thread::sleep(std::time::Duration::from_secs(10));

    //input.start();

    //data::init();
    let mut prod = producer::Producer::new(60.0);
    prod.attach_colorchord(colorchord_channel);
    prod.attach_tempo(tempo_channel);
    prod.attach_onset(onset_channel);
    let p = prod.start();

    // let run = prod.start();
    // run.await;

    join!(p);

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
