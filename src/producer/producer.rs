use crate::audio;
use std::time::{Duration, Instant};
use tokio::time;

pub struct Producer {
    framerate: f64,
    index: u64,
    ticker: time::Interval,
    colorchord_channel: Option<tokio::sync::watch::Receiver<audio::colorchord::NoteResult>>,
}

impl Producer {
    pub fn new(framerate: f64) -> Producer {
        Producer {
            framerate,
            index: 0,
            ticker: time::interval(Duration::from_millis((1000. / framerate) as u64)),
            colorchord_channel: None,
        }
    }
    pub async fn start(&mut self) {
        loop {
            dbg!("start");
            // self.ticker.tick().await;
            tokio::select! {
                val = self.colorchord_channel.as_mut().unwrap().changed()
                ,if self.colorchord_channel.is_some() => {
                    let nw = self.colorchord_channel.as_ref().unwrap().borrow();
                    dbg!(nw);

                }
            }
            dbg!("tick");
        }
    }
    //Attach channels
    pub fn attach_colorchord(
        &mut self,
        chan: tokio::sync::watch::Receiver<audio::colorchord::NoteResult>,
    ) {
        self.colorchord_channel = Some(chan)
    }
}
