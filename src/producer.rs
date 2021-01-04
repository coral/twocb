use std::time::{Duration, Instant};
use tokio::time;

pub struct Producer {
    framerate: f64,
    index: u64,

    ticker: time::Interval,
}

impl Producer {
    pub fn new(framerate: f64) -> Producer {
        Producer {
            framerate,
            index: 0,
            ticker: time::interval(Duration::from_millis((1000. / framerate) as u64)),
        }
    }

    pub async fn start(&mut self) {
        loop {
            self.ticker.tick().await;
            //dbg!("tick");
        }
    }
}
