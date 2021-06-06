use crate::engines::pattern;
use crate::pixels::Pixel;
use crate::producer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    factor: f64,
}
pub struct FoldedDemo {
    s: Settings,
}

impl pattern::Pattern for FoldedDemo {
    fn name(&self) -> String {
        return "foldeddemo".to_string();
    }

    fn process(&mut self, frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>> {
        let dd = frame.square();
        let mut d = vec![[0.0, 0.0, 0.0, 1.0]; frame.mapping.len()];
        let factor = self.s.factor;

        let positions = frame.colorchord.folded.len();
        if positions > 1 {
            for (i, pixel) in frame.mapping.iter().enumerate() {
                let m = (frame.colorchord.folded
                    [((positions as f64) * pixel.position_in_tube()).floor() as usize]
                    .clone() as f64)
                    * factor;
                //dbg!(((positions as f64) * pixel.position_in_tube()).floor() as usize);
                if frame.squarebool() {
                    if pixel.top() {
                        d[i] = [m * (dd * 5.0 + 1.0), m, m, m];
                    }
                } else {
                    if pixel.bottom() {
                        d[i] = [m, 0.0, m, m];
                    }
                }
            }
        }
        return d;
    }

    fn get_state(&self) -> Vec<u8> {
        return serde_json::to_vec(&self.s).unwrap();
    }

    fn set_state(&mut self, data: &[u8]) {
        self.s = serde_json::from_slice(&data).unwrap();
    }
}

impl FoldedDemo {
    pub fn new() -> FoldedDemo {
        FoldedDemo {
            s: Settings { factor: 1.0 },
        }
    }
}
