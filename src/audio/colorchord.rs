use crate::audio;
use log::{debug, info, warn};
use rustchord;
use tokio::sync::broadcast;

pub struct Colorchord {
    dr: crossbeam_channel::Receiver<Vec<f32>>,
    stream_setting: audio::StreamSetting,
    nf: rustchord::Notefinder,

    //chan
    tx: tokio::sync::broadcast::Sender<NoteResult>,
}

#[derive(Debug, Clone)]
pub struct NoteResult {
    notes: Vec<rustchord::Note>,
    folded: Vec<f32>,
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
            let audiodata = self.dr.recv().unwrap();
            self.nf.run(&audiodata);
            let m = NoteResult {
                notes: self.nf.get_notes(),
                folded: self.nf.get_folded().to_owned(),
            };
            self.tx.send(m);
        }
    }
}
