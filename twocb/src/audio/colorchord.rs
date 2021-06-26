use crate::audio;
use crossbeam_channel;
use log::error;
use rustchord;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

pub struct Colorchord {
    dr: crossbeam_channel::Receiver<Vec<f32>>,
    stream_setting: audio::StreamSetting,
    nf: rustchord::Notefinder,

    //chan
    tx: tokio::sync::broadcast::Sender<NoteResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteResult {
    pub notes: Vec<rustchord::Note>,
    pub folded: Vec<f32>,
}

impl Colorchord {
    pub fn new(
        stream_setting: audio::StreamSetting,
        dr: crossbeam_channel::Receiver<Vec<f32>>,
    ) -> Colorchord {
        Colorchord {
            stream_setting,
            dr,
            nf: rustchord::Notefinder::new(stream_setting.sample_rate as i32),
            tx: broadcast::channel(1).0,
        }
    }

    pub fn get_empty() -> NoteResult {
        NoteResult {
            notes: Vec::new(),
            folded: Vec::new(),
        }
    }

    pub fn channel(&self) -> tokio::sync::broadcast::Receiver<NoteResult> {
        return self.tx.subscribe();
    }

    pub fn run(&mut self) {
        loop {
            match self.dr.recv() {
                Ok(audiodata) => {
                    self.nf.run(&audiodata);
                    let m = NoteResult {
                        notes: self.nf.get_notes(),
                        folded: self.nf.get_folded().to_owned(),
                    };
                    let _ = self.tx.send(m);
                }
                Err(e) => {
                    error!("Colorchord recieve error: {}", e);
                }
            }
        }
    }
}
