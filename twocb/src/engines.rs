pub mod dynamic_engine;
pub mod pattern;
pub mod rs_engine;
use async_trait::async_trait;

pub use dynamic_engine::DynamicEngine;
pub use rs_engine::RSEngine;

pub use pattern::Pattern;

#[async_trait]
pub trait Engine {
    fn bootstrap(&mut self) -> anyhow::Result<()>;
    fn list(&self) -> Vec<String>;
    fn instantiate_pattern(&self, name: &str) -> Option<Box<dyn pattern::Pattern + Send>>;
}
