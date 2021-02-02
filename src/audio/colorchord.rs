use crate::audio;
use rustchord;

pub struct Colorchord {
    dr: crossbeam_channel::Receiver<Vec<f32>>,
    stream_setting: audio::StreamSetting,
    nf: rustchord::Notefinder,
}

struct NoteResult {
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
        }
    }
    pub fn run(&mut self) {
        loop {
            let audiodata = self.dr.recv().unwrap();
            self.nf.run(&audiodata);
            let m = NoteResult {
                notes: self.nf.get_notes(),
                folded: self.nf.get_folded().to_owned(),
            };
            for i in m.notes {
                if i.active {
                    dbg!(i);
                }
            }
        }
    }
}
