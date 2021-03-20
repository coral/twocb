use crate::audio;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::sync::watch;
use tokio::time;

pub struct Producer {
    framerate: f64,
    index: u64,
    ticker: time::Interval,

    colorchord_channel: tokio::sync::broadcast::Receiver<audio::colorchord::NoteResult>,
    tempo_channel: tokio::sync::broadcast::Receiver<audio::TempoResult>,
    onset_channel: tokio::sync::broadcast::Receiver<f32>,

    colorchord_data: audio::colorchord::NoteResult,
    tempo_data: audio::TempoResult,

    frame_channel_tx: tokio::sync::watch::Sender<Frame>,
    frame_channel_rx: tokio::sync::watch::Receiver<Frame>,
}

pub struct Frame {}

impl Producer {
    pub fn new(framerate: f64) -> Producer {
        let (tx, mut rx) = watch::channel(Frame {});
        Producer {
            framerate,
            index: 0,
            ticker: tokio::time::interval(Duration::from_millis((1000. / framerate) as u64)),

            colorchord_channel: broadcast::channel(10).1,
            tempo_channel: broadcast::channel(10).1,
            onset_channel: broadcast::channel(10).1,

            colorchord_data: audio::Colorchord::get_empty(),
            tempo_data: audio::Processing::get_empty(),

            frame_channel_tx: tx,
            frame_channel_rx: rx,
        }
    }
    pub async fn start(&mut self) {
        loop {
            tokio::select! {
                _tick = self.ticker.tick() => {
                    self.produce();
                }
            }
        }
    }

    //Internal

    fn produce(&mut self) {
        self.frame_channel_tx.send(Frame {});
    }

    //Attach channels
    pub fn attach_colorchord(
        &mut self,
        chan: tokio::sync::broadcast::Receiver<audio::colorchord::NoteResult>,
    ) {
        self.colorchord_channel = chan;
    }

    pub fn attach_tempo(&mut self, chan: tokio::sync::broadcast::Receiver<audio::TempoResult>) {
        self.tempo_channel = chan;
    }

    pub fn attach_onset(&mut self, chan: tokio::sync::broadcast::Receiver<f32>) {
        self.onset_channel = chan;
    }

    //Get Channels
    pub fn frame_channel(&self) -> tokio::sync::watch::Receiver<Frame> {
        return self.frame_channel_rx.clone();
    }
}
