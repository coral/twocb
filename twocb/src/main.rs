mod api;
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

use clap::{AppSettings, Clap};
use engines::Engine;
use log::error;
use pretty_env_logger;
use std::env;
use std::thread;

use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::sync::oneshot;
use tokio::task;

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

    let mut db = data::DataLayer::new(&cfg.clone().database).unwrap();
    //let mut dbarc = Arc::new(RwLock::new(db.clone()));

    let api_cfg = cfg.clone();
    let api_db = db.clone();
    thread::spawn(move || {
        api::start(
            SocketAddr::new(
                IpAddr::V4(Ipv4Addr::from_str(&api_cfg.api.host).unwrap()),
                api_cfg.api.port,
            ),
            api_db,
        )
        .expect("kek");
    });

    let prc_cfg = cfg.clone();
    let run_db = db.clone();
    //Start the tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            //tokio::spawn(async move { order(order_db).await });
            run(prc_cfg, db).await;
        });
}

pub async fn run(cfg: Arc<config::Config>, db: data::DataLayer) {
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
    let tempo_settings = cfg.audio.tempo.clone();
    let _ap = task::spawn_blocking(move || {
        let mut audioprocessing = audio::Processing::new(audiosetting, stream_processing);
        audioprocessing.set_confidence(tempo_settings.confidence_limit);
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
    let _cr = task::spawn_blocking(move || {
        colorchord.run();
    });

    let mut output = output::OutputManager::new();

    //////////DONE WITH SETUP
    for opc_output in &cfg.endpoints.opc {
        let mut opc = output::OPCOutput::new(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from_str(&opc_output.host).unwrap()),
            opc_output.port as u16,
        ));
        match opc.connect().await {
            Ok(_v) => output.add(Box::new(opc)),
            Err(v) => {
                error!("OPC could not connect: {}", v);
            }
        }
    }

    let mut compositor = Arc::new(tokio::sync::Mutex::new(layers::Compositor::new(db.clone())));
    layers::Controller::new(db.clone(), compositor.clone());
    // let mut dbarc = Arc::new(RwLock::new(db));
    // let layer_controller = layers::Controller::new(dbarc.clone()).bootstrap;

    let mut rse = engines::RSEngine::new();
    rse.bootstrap().unwrap();

    // let mut dse = engines::DynamicEngine::new("files/dynamic/*.js", "files/support/global.js");
    // dse.bootstrap().unwrap();
    // let patterns = dse.list();
    // b!(patterns);

    let stp = layers::Step {
        pattern: rse.instantiate_pattern("foldeddemo").unwrap(),
        blendmode: layers::blending::BlendModes::Add,
    };

    let stp2 = layers::Step {
        pattern: rse.instantiate_pattern("strobe").unwrap(),
        blendmode: layers::blending::BlendModes::Add,
    };

    // let stp3 = layers::Step {
    //     pattern: dse.instantiate_pattern("first.js").unwrap(),
    //     blendmode: layers::blending::BlendModes::Add,
    // };

    let lnk = layers::Link::create(String::from("firstExperince"), vec![stp, stp2]);

    compositor.lock().await.add_link(lnk).await;

    let map =
        pixels::Mapping::load_from_file("files/mappings/v6.json").expect("Could not load mapping");
    let mut prod = producer::Producer::new(60.0, map);

    prod.attach_colorchord(colorchord_channel);
    prod.attach_tempo(tempo_channel);
    prod.attach_onset(onset_channel);
    let mut framechan = prod.frame_channel();
    tokio::spawn(async move {
        tokio::join!(prod.start());
    });

    loop {
        let frame_data = framechan.recv().await.unwrap();

        let rst = compositor.lock().await.render(frame_data).await;
        output.write(&rst);
    }
}
