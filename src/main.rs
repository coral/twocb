mod audio;
mod data;
mod engines;
mod layers;
mod output;
mod patterns;
mod pixels;
mod producer;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{thread, time};

use clap::{AppSettings, Clap};
use engines::Engine;
use log::{error, info, warn};
use output::Adapter;
use pretty_env_logger;
use std::env;
use tokio::sync::oneshot;
use tokio::task;

use std::time::{Duration, Instant};

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "files/config.json")]
    config: String,
}

fn main() {
    let opts: Opts = Opts::parse();
    println!("Value for config: {}", opts.config);

    env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();

    //Data layer
    let mut db = data::DataLayer::new("files/settings.db").unwrap();
    db.woo();

    //Start the tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            run().await;
        })
}

pub async fn run() {
    ////AUDIOSHIT

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
    let ap = task::spawn_blocking(move || {
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
    let cr = task::spawn_blocking(move || {
        colorchord.run();
    });

    //////////DONE WITH SETUP

    let mut opc = output::OPCOutput::new(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        7890,
    ));
    match opc.connect().await {
        Ok(v) => {}
        Err(v) => {
            error!("OPC could not connect: {}", v);
        }
    }

    let mut rse = engines::RSEngine::new();
    rse.bootstrap().unwrap();

    //let mut dse = engines::DynamicEngine::new("files/dynamic/*.js", "files/support/global.js");
    //dse.bootstrap().unwrap();
    //let patterns = dse.list();
    //dbg!(patterns);

    let stp = layers::Step {
        pattern: rse.instantiate_pattern("foldeddemo").unwrap(),
        blendmode: layers::blending::BlendModes::Add,
    };

    let lnk = layers::Link::create(String::from("firstExperince"), vec![stp]);

    let mut manager = layers::Manager::new();

    manager.add_link(lnk);

    let mut prod = producer::Producer::new(60.0);

    prod.attach_colorchord(colorchord_channel);
    prod.attach_tempo(tempo_channel);
    prod.attach_onset(onset_channel);
    let mut framechan = prod.frame_channel();
    tokio::spawn(async move {
        tokio::join!(prod.start());
    });

    loop {
        let frame_data = framechan.recv().await;
        match frame_data {
            Ok(frame_data) => {
                let rst = manager.render(frame_data).await;
                opc.write(rst);
            }
            Err(_) => {}
        }
    }
}
// pub async fn main() {

//     // let join = task::spawn(async {
//     //     let map = pixels::Mapping::load_from_file("files/mappings/v6.json").unwrap();
//     //     let mut p = patterns::dynamic::Pattern::create("examples/debug.js", map.clone());
//     //     p.load();
//     //     p.setup();
//     //     p.register();
//     //     p.process()
//     // });
//     // let mut p = patterns::dynamic::Pattern::create("examples/debug.js", map.clone());
//     // p.load();
//     // p.setup();
//     // p.register();
//     // let result = join.await;
//     // dbg!(result);
//     // let now = Instant::now();
//     // let invocations = 10000;
//     // for _ in 0..invocations {
//     //     let mut m = p.process();
//     //     m[0] = 1.0;
//     //     dbg!(m);
//     // }
//     // println!(
//     //     "Time: {}ms, {} invocations per second,",
//     //     now.elapsed().as_millis(),
//     //     invocations as f64 / (now.elapsed().as_millis() as f64 / 1000.)
//     // );
// }
