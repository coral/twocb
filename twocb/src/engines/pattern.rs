use crate::producer;
use std::sync::Arc;
use vecmath;

pub trait Pattern {
    fn name(&self) -> String;

    fn process(&mut self, frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>>;
}
