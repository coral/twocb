use crate::audio;
use log::{debug, warn};
use std::f64::consts::{FRAC_PI_2, PI};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time;

#[allow(dead_code)]
pub struct Producer {
    framerate: f64,
    index: u64,
    tempo: f64,
    ticker: time::Interval,

    start: Instant,
    cycletimer: Instant,
    last_frame: Instant,

    colorchord_channel: tokio::sync::broadcast::Receiver<audio::colorchord::NoteResult>,
    tempo_channel: tokio::sync::broadcast::Receiver<audio::TempoResult>,
    tempo_enabled: bool,
    onset_channel: tokio::sync::broadcast::Receiver<f32>,

    colorchord_data: audio::colorchord::NoteResult,
    tempo_data: audio::TempoResult,

    frame_channel_tx: tokio::sync::broadcast::Sender<Frame>,
}

impl Producer {
    pub fn new(framerate: f64) -> Producer {
        Producer {
            framerate,
            index: 0,
            tempo: (60.0 / 120.0),
            ticker: tokio::time::interval(Duration::from_millis((1000. / framerate) as u64)),

            start: Instant::now(),
            cycletimer: Instant::now(),
            last_frame: Instant::now(),

            colorchord_channel: broadcast::channel(1).1,
            tempo_channel: broadcast::channel(10).1,
            tempo_enabled: false,
            onset_channel: broadcast::channel(10).1,

            colorchord_data: audio::Colorchord::get_empty(),
            tempo_data: audio::Processing::get_empty(),

            frame_channel_tx: broadcast::channel(1).0,
        }
    }
    pub async fn start(&mut self) {
        loop {
            tokio::select! {
                _tick = self.ticker.tick() => {
                    self.produce();
                }
                Ok(v) = self.colorchord_channel.recv() => {
                    self.colorchord_data = v;
                }
                Ok(v) = self.tempo_channel.recv() => {
                    if self.tempo_enabled {
                        self.sync_tempo(v);
                    }
                }
                else => {
                }
            }
            self.index = self.index + 1;
        }
    }

    //Internal

    fn produce(&mut self) {
        if self
            .frame_channel_tx
            .send(Frame {
                framerate: self.framerate,
                index: self.index,

                delta: self.last_frame.elapsed().as_millis() as f64,
                phase: self.get_phase(),

                colorchord: self.colorchord_data.clone(),
                tempo: self.tempo_data.clone(),
            })
            .is_err()
        {
            warn!("Could not feed framechannel");
        }

        self.last_frame = Instant::now();
    }

    fn sync_tempo(&mut self, t: audio::processing::TempoResult) {
        debug!("[TEMPO] BPM: {0:.2}, Conf: {1:.2}", t.bpm, t.confidence);
        self.cycletimer = t.time;
        self.tempo = 120.0 / (t.bpm as f64);

        self.tempo_data = t;
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
        self.tempo_enabled = true;
    }

    pub fn attach_onset(&mut self, chan: tokio::sync::broadcast::Receiver<f32>) {
        self.onset_channel = chan;
    }

    //Get Channels
    pub fn frame_channel(&self) -> tokio::sync::broadcast::Receiver<Frame> {
        return self.frame_channel_tx.subscribe();
    }

    fn get_phase(&self) -> f64 {
        self.cycletimer.elapsed().as_secs_f64() / self.tempo % 1.0
    }

    // Settings
    pub fn _follow_tempo(&mut self, follow: bool) {
        self.tempo_enabled = follow;
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub framerate: f64,
    pub index: u64,

    pub delta: f64,
    pub phase: f64,

    pub colorchord: audio::colorchord::NoteResult,
    pub tempo: audio::TempoResult,
}

#[allow(dead_code)]
impl Frame {
    pub fn sin(&self, cycle: f64, offset: f64) -> f64 {
        ((self.phase + offset) * PI * cycle).sin()
    }

    pub fn cos(&self, cycle: f64, offset: f64) -> f64 {
        ((self.phase + offset) * PI * cycle).cos()
    }

    pub fn square(&self) -> f64 {
        if self.phase <= 0.5 {
            return 0.0;
        } else {
            return 1.0;
        }
    }

    pub fn triangle(&self, cycle: f64) -> f64 {
        (self.sin(cycle, 0.0)).acos() / FRAC_PI_2
    }
}
