use crate::engines::pattern;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Strobe {}

impl pattern::Pattern for Strobe {
    fn name(&self) -> String {
        return "strobe".to_string();
    }

    fn process(&mut self) -> Vec<vecmath::Vector4<f64>> {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as f64;
        return vec![[t.sin(), t.cos(), 1.0, 1.0]; 700];
    }
}

impl Strobe {
    pub fn new() -> Strobe {
        Strobe {}
    }

    pub fn name() -> String {
        return "strobe".to_string();
    }
}
