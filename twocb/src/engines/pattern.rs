use crate::producer;
use std::sync::Arc;
use tokio::sync::mpsc;
use vecmath;

pub trait Pattern {
    fn name(&self) -> String;

    fn process(&mut self, frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>>;

    fn get_state(&self) -> Vec<u8>;
    fn set_state(&mut self, data: &[u8]);
}
