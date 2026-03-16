mod api;
mod audio;
mod config;
mod controller;
mod data;
mod engines;
mod layers;
mod world_state;

mod output;
mod pixels;
mod producer;
mod rtc;
mod ws;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use clap::Parser;
use log::error;
use pretty_env_logger;
use std::env;
use std::sync::Arc;
use std::thread;
use tokio::sync::Mutex;
use tokio::sync::{broadcast, oneshot};
use tokio::task;

use rtc::{PeerManager, PixelBroadcast};

#[derive(Parser)]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "files/config.json")]
    config: String,
}

fn main() {
    //Start the tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            bootstrap().await;
        });
}

pub async fn bootstrap() {
    unsafe { env::set_var("RUST_LOG", "debug") };
    pretty_env_logger::init();

    let opts: Opts = Opts::parse();
    let cfg = match config::load_config(&opts.config) {
        Ok(cfg) => cfg,
        Err(error) => {
            error!("Could not load the config file: {:?}", error);
            std::process::exit(2);
        }
    };

    let db = data::DataLayer::new(&cfg.clone().database).unwrap();

    let map = match pixels::Mapping::load_from_file(
        &std::path::Path::new("files/mappings/").join(&cfg.mapping),
    ) {
        Ok(map) => map,
        Err(e) => {
            panic!("Could not load mapping: {}", e);
        }
    };

    let compositor = Arc::new(tokio::sync::Mutex::new(
        layers::compositor::Compositor::new(),
    ));
    let mut ctrl = controller::Controller::new(db.clone(), compositor.clone(), map.clone());
    ctrl.bootstrap().await;

    controller::Controller::watch_state_changes(db.clone(), compositor.clone());

    let ctrl = Arc::new(tokio::sync::Mutex::new(ctrl));

    // WebRTC pixel broadcast
    let pixel_broadcast = PixelBroadcast::new();
    let peer_manager = Arc::new(Mutex::new(PeerManager::new()));

    // State notification channel for WebSocket
    let (state_tx, _) = broadcast::channel::<String>(64);

    let api_cfg = cfg.clone();
    let api_state = api::ApiState {
        db: db.clone(),
        ctrl: ctrl.clone(),
        peer_manager: peer_manager.clone(),
        pixel_broadcast: pixel_broadcast.clone(),
        state_tx: state_tx.clone(),
        config: cfg.clone(),
    };

    thread::spawn(move || {
        api::start(
            SocketAddr::new(
                IpAddr::V4(Ipv4Addr::from_str(&api_cfg.api.host).unwrap()),
                api_cfg.api.port,
            ),
            api_state,
        )
        .expect("API server failed to start");
    });

    let prc_cfg = cfg.clone();
    let cmps = compositor.clone();

    run(prc_cfg, cmps, map.clone(), pixel_broadcast).await;
}

use std::str::FromStr;

pub async fn run(
    cfg: Arc<config::Config>,
    compositor: Arc<Mutex<layers::compositor::Compositor>>,
    mapping: Vec<pixels::Pixel>,
    pixel_broadcast: PixelBroadcast,
) {
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
    for ep in &cfg.endpoints {
        let pixel_count = ep.end - ep.start;
        match ep.protocol {
            config::Protocol::Opc => {
                let addr = SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::from_str(&ep.host).unwrap()),
                    ep.port,
                );
                let mut opc = output::OPCOutput::new(addr, pixel_count);
                match opc.connect().await {
                    Ok(_) => output.add(Box::new(opc), ep.start, ep.end),
                    Err(e) => error!("OPC connect failed {}: {}", addr, e),
                }
            }
            config::Protocol::Ddp => {
                let addr = format!("{}:{}", ep.host, ep.port);
                match output::DDPOutput::new(&addr, pixel_count) {
                    Ok(ddp) => output.add(Box::new(ddp), ep.start, ep.end),
                    Err(e) => error!("DDP init failed {}: {}", addr, e),
                }
            }
        }
    }

    let mut prod = producer::Producer::new(cfg.fps, mapping);

    prod.attach_colorchord(colorchord_channel);
    prod.attach_tempo(tempo_channel);
    prod.attach_onset(onset_channel);
    let mut framechan = prod.frame_channel();
    tokio::spawn(async move {
        tokio::join!(prod.start());
    });

    let mut frame_counter: u32 = 0;

    loop {
        match framechan.recv().await {
            Ok(frame) => {
                let rst = compositor.lock().await.render(frame).await;
                output.write(&rst);

                // Broadcast every 3rd frame to WebRTC peers (~33 FPS)
                frame_counter = frame_counter.wrapping_add(1);
                if frame_counter % 3 == 0 {
                    let packed = PixelBroadcast::pack_frame(frame_counter, &rst);
                    let _ = pixel_broadcast.pixel_tx.send(Arc::new(packed));
                }
            }
            Err(e) => {
                error!("{}", e)
            }
        };
    }
}
