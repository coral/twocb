use crate::audio;
use aubio_rs::{Onset, Tempo, FFT};
use std::rc::Rc;
use std::time::Instant;
use tokio::sync::watch;

pub struct Processing {
    dr: crossbeam_channel::Receiver<Vec<f32>>,
    stream_setting: audio::StreamSetting,
    tempo: Tempo,
    onset: Onset,
    fft: FFT,
    //chan
    tempo_tx: tokio::sync::watch::Sender<TempoResult>,
    tempo_rx: tokio::sync::watch::Receiver<TempoResult>,

    onset_tx: tokio::sync::watch::Sender<f32>,
    onset_rx: tokio::sync::watch::Receiver<f32>,
}

#[derive(Debug, Clone)]
pub struct TempoResult {
    bpm: f32,
    confidence: f32,
    period: f32,
    time: Instant,
}

impl Processing {
    pub fn new(
        stream_setting: audio::StreamSetting,
        dr: crossbeam_channel::Receiver<Vec<f32>>,
    ) -> Processing {
        let (tempo_tx, mut tempo_rx) = watch::channel(TempoResult {
            bpm: 0.0,
            confidence: 0.0,
            period: 0.0,
            time: Instant::now(),
        });

        let (onset_tx, mut onset_rx) = watch::channel(0.0);

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
            tempo_rx: tempo_rx,

            onset_tx: onset_tx,
            onset_rx: onset_rx,
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

    pub fn tempo_channel(&self) -> tokio::sync::watch::Receiver<TempoResult> {
        return self.tempo_rx.clone();
    }

    pub fn onset_channel(&self) -> tokio::sync::watch::Receiver<f32> {
        return self.onset_rx.clone();
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
