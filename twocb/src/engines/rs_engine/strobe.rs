use crate::engines::pattern;
use crate::producer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    lit: bool,
}

pub struct Strobe {
    s: Settings,
}

impl pattern::Pattern for Strobe {
    fn name(&self) -> String {
        return "strobe".to_string();
    }

    fn process(&mut self, frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>> {
        let mut d = vec![[0.0, 0.0, 0.0, 1.0]; frame.mapping.len()];
        for (i, pixel) in frame.mapping.iter().enumerate() {
            if frame.squarebool() {
                if pixel.front() {
                    d[i] = [1.0, 1.0, 1.0, 1.0];
                }
            } else {
                if pixel.back() {
                    d[i] = [1.0, 1.0, 1.0, 1.0];
                }
            }
        }
        return d;
    }

    fn get_state(&self) -> Vec<u8> {
        return serde_json::to_vec(&self.s).unwrap();
    }

    fn set_state(&mut self, data: Vec<u8>) {
        self.s = serde_json::from_slice(&data).unwrap();
    }
}

impl Strobe {
    pub fn new() -> Strobe {
        Strobe {
            s: Settings { lit: false },
        }
    }
}
