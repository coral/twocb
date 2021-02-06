use crate::audio;
use rustchord;
use tokio::sync::watch;

pub struct Colorchord {
    dr: crossbeam_channel::Receiver<Vec<f32>>,
    stream_setting: audio::StreamSetting,
    nf: rustchord::Notefinder,

    //chan
    tx: tokio::sync::watch::Sender<NoteResult>,
    rx: tokio::sync::watch::Receiver<NoteResult>,
}

#[derive(Debug)]
pub struct NoteResult {
    notes: Vec<rustchord::Note>,
    folded: Vec<f32>,
}

impl Colorchord {
    pub fn new(
        stream_setting: audio::StreamSetting,
        dr: crossbeam_channel::Receiver<Vec<f32>>,
    ) -> Colorchord {
        let (tx, mut rx) = watch::channel(NoteResult {
            notes: Vec::new(),
            folded: Vec::new(),
        });
        Colorchord {
            stream_setting,
            dr,
            nf: rustchord::Notefinder::new(stream_setting.sample_rate as i32),
            tx: tx,
            rx: rx,
        }
    }

    pub fn channel(&self) -> tokio::sync::watch::Receiver<NoteResult> {
        return self.rx.clone();
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
