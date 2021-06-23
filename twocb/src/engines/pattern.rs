use crate::producer;
use async_trait::async_trait;
use std::sync::Arc;
use vecmath;

#[async_trait]
pub trait Pattern: Send {
    fn name(&self) -> String;

    async fn process(&mut self, frame: Arc<producer::Frame>) -> Vec<vecmath::Vector4<f64>>;

    fn get_state(&self) -> Vec<u8>;
    fn set_state(&mut self, data: &[u8]);
}
