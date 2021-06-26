use crate::audio;
use aubio::{Onset, Tempo, FFT};
use log::error;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::sync::broadcast;

pub struct Processing {
    dr: crossbeam_channel::Receiver<Vec<f32>>,
    stream_setting: audio::StreamSetting,
    tempo: Tempo,
    confidence_limit: f32,
    onset: Onset,
    fft: FFT,
    //chan
    tempo_tx: tokio::sync::broadcast::Sender<TempoResult>,
    onset_tx: tokio::sync::broadcast::Sender<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoResult {
    pub bpm: f32,
    pub confidence: f32,
    pub period: f32,
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
                aubio::OnsetMode::SpecFlux,
                stream_setting.buffer_size as usize,
                512,
                stream_setting.sample_rate,
            )
            .unwrap(),

            confidence_limit: 0.2,

            //ONSET
            onset: Onset::new(
                aubio::OnsetMode::SpecFlux,
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
        }
    }

    pub fn tempo_channel(&self) -> tokio::sync::broadcast::Receiver<TempoResult> {
        return self.tempo_tx.subscribe();
    }

    pub fn onset_channel(&self) -> tokio::sync::broadcast::Receiver<f32> {
        return self.onset_tx.subscribe();
    }

    pub fn set_confidence(&mut self, conf: f32) {
        self.confidence_limit = conf;
    }

    pub fn run(&mut self) {
        loop {
            let audiodata = self.dr.recv().unwrap();

            //Tempo
            match self.tempo.do_result(&audiodata) {
                Ok(tempodata) => {
                    if tempodata > 0.0 && self.tempo.get_confidence() > 0.03 {
                        let t = TempoResult {
                            bpm: self.tempo.get_bpm(),
                            confidence: self.tempo.get_confidence(),
                            period: self.tempo.get_period_s(),
                        };
                        match self.tempo_tx.send(t) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Tempo channel send fail: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Tempo fail: {}", e);
                }
            }

            //ONSET

            match self.onset.do_result(&audiodata) {
                Ok(onsetdata) => {
                    if onsetdata > 0.0 {
                        match self.onset_tx.send(onsetdata) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Onset channel send fail: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Onset result error: {}", e);
                }
            }

            //FFT
            //let fftdata = self.fft.do_(input: I, spectrum: O)
        }
    }
}
