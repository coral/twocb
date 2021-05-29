use crate::engines::pattern;
use crate::producer;
use std::sync::Arc;

pub struct Strobe {
    lit: bool,
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
}

impl Strobe {
    pub fn new() -> Strobe {
        Strobe { lit: false }
    }
}
