use crate::audio;
use aubio_rs::{Onset, Tempo};
use rustchord;

pub struct Processing {
    dr: crossbeam_channel::Receiver<Vec<f32>>,
    stream_setting: audio::StreamSetting,

    tempo: Tempo,
}

impl Processing {
    pub fn new(
        stream_setting: audio::StreamSetting,
        dr: crossbeam_channel::Receiver<Vec<f32>>,
    ) -> Processing {
        Processing {
            stream_setting,
            dr,
            tempo: Tempo::new(
                aubio_rs::OnsetMode::SpecFlux,
                stream_setting.buffer_size as usize,
                256,
                stream_setting.sample_rate,
            )
            .unwrap(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let audiodata = self.dr.recv().unwrap();

            ///TEMPO
            let tempodata = self.tempo.do_result(audiodata).unwrap();

            if tempodata > 0.0 {
                // println!("Tempo: {:?}", self.tempo.get_bpm());
                // println!("Confidence: {:?}", self.tempo.get_confidence());
            }
        }
    }
}
