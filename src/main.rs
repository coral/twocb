mod audio;
mod config;
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
use std::rc::Rc;
use std::str::FromStr;
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
    env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();

    let opts: Opts = Opts::parse();
    let cfg = match config::load_config(&opts.config) {
        Ok(cfg) => cfg,
        Err(error) => {
            error!("Could not load the config file: {:?}", error);
            std::process::exit(2);
        }
    };

    //Start the tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            run(cfg).await;
        })
}

pub async fn run(cfg: config::Config) {
    //Data layer
    let mut db = data::DataLayer::new(&cfg.database).unwrap();
    db.woo();

    ////AUDIOSHIT

    let audiosetting = audio::StreamSetting {
        sample_rate: cfg.audio.sample_rate,
        buffer_size: cfg.audio.buffer_size,
        channels: cfg.audio.channels,
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

    let mut output = output::OutputManager::new();

    //////////DONE WITH SETUP
    for opc_output in cfg.endpoints.opc {
        let mut opc = output::OPCOutput::new(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from_str(&opc_output.host).unwrap()),
            opc_output.port as u16,
        ));
        match opc.connect().await {
            Ok(v) => output.add(Box::new(opc)),
            Err(v) => {
                error!("OPC could not connect: {}", v);
            }
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
        let frame_data = framechan.recv().await.unwrap();

        let rst = manager.render(frame_data).await;
        output.write(&rst);
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
