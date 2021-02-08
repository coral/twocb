use crate::audio;
use std::time::{Duration, Instant};
use tokio::time;

pub struct Producer {
    framerate: f64,
    index: u64,
    ticker: time::Interval,

    colorchord_channel: Option<tokio::sync::watch::Receiver<audio::colorchord::NoteResult>>,
    tempo_channel: Option<tokio::sync::watch::Receiver<audio::TempoResult>>,
    onset_channel: Option<tokio::sync::watch::Receiver<f32>>,

    colorchord_data: audio::colorchord::NoteResult,
    tempo_data: audio::TempoResult,
}

impl Producer {
    pub fn new(framerate: f64) -> Producer {
        Producer {
            framerate,
            index: 0,
            ticker: tokio::time::interval(Duration::from_millis((1000. / framerate) as u64)),

            colorchord_channel: None,
            tempo_channel: None,
            onset_channel: None,

            colorchord_data: audio::Colorchord::get_empty(),
            tempo_data: audio::Processing::get_empty(),
        }
    }
    pub async fn start(&mut self) {
        loop {
            tokio::select! {
                _val = self.colorchord_channel.as_mut().unwrap().changed()
                ,if self.colorchord_channel.is_some() => {
                    let nw = self.colorchord_channel.as_ref().unwrap().borrow();
                    self.colorchord_data = nw.clone();
                }

                _val = self.tempo_channel.as_mut().unwrap().changed()
                , if self.tempo_channel.is_some() => {
                    let nw = self.tempo_channel.as_ref().unwrap().borrow();
                    self.tempo_data = nw.clone();
                }

                _val = self.onset_channel.as_mut().unwrap().changed()
                , if self.onset_channel.is_some() => {
                    let nw = self.onset_channel.as_ref().unwrap().borrow();
                    println!("ONSET: {:.1}", *nw);
                }

                _tick = self.ticker.tick() => {
                    self.produce().await;
                }
            }
        }
    }

    //Internal

    async fn produce(&mut self) {
        dbg!(&self.colorchord_data);
    }

    //Attach channels
    pub fn attach_colorchord(
        &mut self,
        chan: tokio::sync::watch::Receiver<audio::colorchord::NoteResult>,
    ) {
        self.colorchord_channel = Some(chan)
    }

    pub fn attach_tempo(&mut self, chan: tokio::sync::watch::Receiver<audio::TempoResult>) {
        self.tempo_channel = Some(chan)
    }

    pub fn attach_onset(&mut self, chan: tokio::sync::watch::Receiver<f32>) {
        self.onset_channel = Some(chan)
    }
}
