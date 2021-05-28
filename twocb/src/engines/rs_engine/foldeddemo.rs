use crate::engines::pattern;
use crate::producer;
use std::sync::Arc;

pub struct FoldedDemo {}

impl pattern::Pattern for FoldedDemo {
    fn name(&self) -> String {
        return "foldeddemo".to_string();
    }

    fn process(&mut self, frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>> {
        let dd = frame.square();
        let mut d = vec![[dd, dd, dd, 1.0]; 864];
        let mut i = 0;
        for x in &frame.colorchord.folded {
            let m = (x.clone() * 10.0) as f64;
            d[i] = [m, m, m, 1.0];
            i = i + 1;
        }
        return d;
    }
}

impl FoldedDemo {
    pub fn new() -> FoldedDemo {
        FoldedDemo {}
    }
}
