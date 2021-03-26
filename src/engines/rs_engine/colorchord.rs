use crate::engines::pattern;
use crate::producer;
use std::sync::Arc;

pub struct Colorchord {}

impl pattern::Pattern for Colorchord {
    fn name(&self) -> String {
        return "strobe".to_string();
    }

    fn process(&mut self, frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>> {
        let mut d = vec![[0.0, 0.0, 0.0, 1.0]; 864];
        let mut i = 0;
        for x in &frame.colorchord.folded {
            let m = (x.clone() * 10.0) as f64;
            d[i] = [m, m, m, 1.0];
            i = i + 1;
        }
        return d;
    }
}

impl Colorchord {
    pub fn new() -> Colorchord {
        Colorchord {}
    }

    pub fn name() -> String {
        return "colorchord".to_string();
    }
}
