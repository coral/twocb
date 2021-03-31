use crate::audio;
use aubio_rs::{Onset, Tempo, FFT};
use std::time::Instant;
use tokio::sync::broadcast;

pub struct Processing {
    dr: crossbeam_channel::Receiver<Vec<f32>>,
    stream_setting: audio::StreamSetting,
    tempo: Tempo,
    onset: Onset,
    fft: FFT,
    //chan
    tempo_tx: tokio::sync::broadcast::Sender<TempoResult>,
    onset_tx: tokio::sync::broadcast::Sender<f32>,
}

#[derive(Debug, Clone)]
pub struct TempoResult {
    pub bpm: f32,
    pub confidence: f32,
    pub period: f32,
    pub time: Instant,
}

impl Processing {
    pub fn new(
        stream_setting: audio::StreamSetting,
        dr: crossbeam_channel::Receiver<Vec<f32>>,
    ) -> Processing {
        let tempo_tx = broadcast::channel(1).0;

        let onset_tx = broadcast::channel(1).0;

        Processing {
            stream_setting,
            dr,
            //TEMPO
            tempo: Tempo::new(
                aubio_rs::OnsetMode::SpecFlux,
                stream_setting.buffer_size as usize,
                512,
                stream_setting.sample_rate,
            )
            .unwrap(),

            //ONSET
            onset: Onset::new(
                aubio_rs::OnsetMode::SpecFlux,
                stream_setting.buffer_size as usize,
                512,
                stream_setting.sample_rate,
            )
            .unwrap(),

            //FFT
            fft: FFT::new(256).unwrap(),

            //Channel stuff
            tempo_tx: tempo_tx,

            onset_tx: onset_tx,
        }
    }

    pub fn get_empty() -> TempoResult {
        TempoResult {
            bpm: 120.0,
            confidence: 0.2,
            period: 0.0,
            time: Instant::now(),
        }
    }

    pub fn tempo_channel(&self) -> tokio::sync::broadcast::Receiver<TempoResult> {
        return self.tempo_tx.subscribe();
    }

    pub fn onset_channel(&self) -> tokio::sync::broadcast::Receiver<f32> {
        return self.onset_tx.subscribe();
    }

    pub fn run(&mut self) {
        loop {
            let audiodata = self.dr.recv().unwrap();

            ///TEMPO
            let tempodata = self.tempo.do_result(&audiodata).unwrap();
            if tempodata > 0.0 {
                let t = TempoResult {
                    bpm: self.tempo.get_bpm(),
                    confidence: self.tempo.get_confidence(),
                    period: self.tempo.get_period_s(),
                    time: Instant::now(),
                };
                self.tempo_tx.send(t).unwrap();
            }

            //ONSET
            let onsetdata = self.onset.do_result(&audiodata).unwrap();
            if onsetdata > 0.0 {
                self.onset_tx.send(onsetdata).unwrap();
            }

            //FFT
            //let fftdata = self.fft.do_(input: I, spectrum: O)
        }
    }
}
